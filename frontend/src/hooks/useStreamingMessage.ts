import { useCallback, useEffect, useRef, useState } from "react";
import { streamMessage } from "../lib/api";
import type { Message } from "../types/api";

interface UseStreamingMessageResult {
  messages: Message[];
  streamingText: string;
  isStreaming: boolean;
  error: string | null;
  sendMessage: (content: string) => void;
}

interface SSEEvent {
  event: string | null;
  data: string;
}

async function parseSSEStream(
  reader: ReadableStreamDefaultReader<Uint8Array>,
  onToken: (text: string) => void,
  onDone: (messageId: string) => void,
  onError: (message: string) => void,
): Promise<void> {
  const decoder = new TextDecoder();
  let buffer = "";
  let event: string | null = null;
  let data = "";

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    buffer += decoder.decode(value, { stream: true });
    const lines = buffer.split("\n");
    buffer = lines.pop() || "";

    for (const line of lines) {
      if (line === "") {
        if (event && data) {
          try {
            const parsed = JSON.parse(data);
            if (event === "token" && parsed.text) {
              onToken(parsed.text);
            } else if (event === "done" && parsed.message_id) {
              onDone(parsed.message_id);
            } else if (event === "error" && parsed.message) {
              onError(parsed.message);
            }
          } catch {
            onError("Failed to parse SSE event");
          }
        }
        event = null;
        data = "";
      } else if (line.startsWith("event: ")) {
        event = line.slice(7);
      } else if (line.startsWith("data: ")) {
        data += (data ? "\n" : "") + line.slice(6);
      }
    }
  }

  if (event && data) {
    try {
      const parsed = JSON.parse(data);
      if (event === "token" && parsed.text) {
        onToken(parsed.text);
      } else if (event === "done" && parsed.message_id) {
        onDone(parsed.message_id);
      } else if (event === "error" && parsed.message) {
        onError(parsed.message);
      }
    } catch {
      onError("Failed to parse final SSE event");
    }
  }
}

export function useStreamingMessage(
  conversationId: string,
  initialMessages: Message[],
): UseStreamingMessageResult {
  const [messages, setMessages] = useState<Message[]>(initialMessages);
  const [streamingText, setStreamingText] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const hydrated = useRef(false);

  useEffect(() => {
    if (!hydrated.current && initialMessages.length > 0) {
      hydrated.current = true;
      setMessages(initialMessages);
    }
  }, [initialMessages]);

  const sendMessage = useCallback(
    async (content: string) => {
      setError(null);
      setStreamingText("");
      setIsStreaming(true);

      const userMessage: Message = {
        id: `user-${Date.now()}`,
        role: "user",
        content,
      };
      setMessages((prev) => [...prev, userMessage]);

      let fullText = "";

      try {
        const res = await streamMessage(conversationId, content);
        if (!res.ok || !res.body) {
          throw new Error(`${res.status}: ${res.statusText}`);
        }

        const reader = res.body.getReader();

        await parseSSEStream(
          reader,
          (text) => {
            fullText += text;
            setStreamingText((prev) => prev + text);
          },
          (messageId) => {
            setMessages((prev) => [
              ...prev,
              {
                id: messageId,
                role: "assistant",
                content: fullText,
              },
            ]);
            setStreamingText("");
            setIsStreaming(false);
          },
          (errorMessage) => {
            setError(errorMessage);
            setIsStreaming(false);
          },
        );
      } catch (err) {
        const message = err instanceof Error ? err.message : "Unknown error";
        setError(message);
        setIsStreaming(false);
      }
    },
    [conversationId],
  );

  return { messages, streamingText, isStreaming, error, sendMessage };
}
