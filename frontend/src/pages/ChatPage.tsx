import { useEffect, useRef, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import ReactMarkdown from "react-markdown";
import { getConversation, getPersonaById } from "../lib/api";
import { useStreamingMessage } from "../hooks/useStreamingMessage";
import { SummaryDrawer } from "./SummaryDrawer";
import type { Message, Persona } from "../types/api";

function nameToHue(name: string): number {
  let sum = 0;
  for (let i = 0; i < name.length; i++) {
    sum += name.charCodeAt(i);
  }
  return sum % 360;
}

function PortraitSquare({
  name,
  hue,
  className = "",
}: {
  name: string;
  hue: number;
  className?: string;
}) {
  const initials = name
    .split(" ")
    .filter((w) => /^[A-Z]/.test(w))
    .slice(0, 2)
    .map((w) => w[0])
    .join("");
  return (
    <div
      className={`relative overflow-hidden rounded-xl ring-1 ring-[#E8E0D0] shadow-sm ${className}`}
      style={{
        aspectRatio: "1 / 1",
        background: `linear-gradient(140deg, oklch(0.92 0.04 ${hue}) 0%, oklch(0.72 0.07 ${hue}) 100%)`,
      }}
      aria-hidden="true"
    >
      <div
        className="absolute inset-0 opacity-[0.18] mix-blend-multiply"
        style={{
          backgroundImage:
            "repeating-linear-gradient(45deg, rgba(0,0,0,0.4) 0 1px, transparent 1px 8px)",
        }}
      ></div>
      <div
        className="absolute inset-x-0 bottom-0 h-1/3"
        style={{
          background: "linear-gradient(180deg, transparent, rgba(0,0,0,0.18))",
        }}
      ></div>
      <div className="absolute inset-0 grid place-items-center">
        <span
          className="font-mono tracking-widest text-[#2C2C2C]/55"
          style={{ fontSize: "clamp(28px, 7vw, 72px)" }}
        >
          {initials || "··"}
        </span>
      </div>
      <div className="absolute bottom-2 left-2 right-2 flex items-center justify-between font-mono text-[10px] uppercase tracking-wider text-white/80">
        <span>portrait</span>
        <span>1:1</span>
      </div>
    </div>
  );
}

function MiniAvatar({
  name,
  hue,
  size = 28,
}: {
  name: string;
  hue: number;
  size?: number;
}) {
  const initials = name
    .split(" ")
    .filter((w) => /^[A-Z]/.test(w))
    .slice(0, 2)
    .map((w) => w[0])
    .join("");
  return (
    <div
      className="relative shrink-0 overflow-hidden rounded-full ring-1 ring-[#E8E0D0]"
      style={{
        width: size,
        height: size,
        background: `linear-gradient(140deg, oklch(0.92 0.04 ${hue}) 0%, oklch(0.78 0.06 ${hue}) 100%)`,
      }}
      aria-hidden="true"
    >
      <div
        className="absolute inset-0 grid place-items-center font-mono text-[#2C2C2C]/60"
        style={{ fontSize: Math.round(size * 0.34) }}
      >
        {initials || "··"}
      </div>
    </div>
  );
}

function BlinkingCursor() {
  return (
    <span
      aria-hidden="true"
      className="ml-0.5 inline-block h-[1em] w-[2px] translate-y-[2px] bg-[#8B6F47] align-middle"
      style={{ animation: "animus-blink 1s steps(1) infinite" }}
    ></span>
  );
}

function MessageRow({
  message,
  isStreaming,
  personaHue,
  personaName,
}: {
  message: { id: string; role: "user" | "assistant"; content: string };
  isStreaming: boolean;
  personaHue: number;
  personaName: string;
}) {
  const isUser = message.role === "user";
  return (
    <div className={`flex w-full ${isUser ? "justify-end" : "justify-start"}`}>
      {!isUser && (
        <div className="mr-2 mt-1">
          <MiniAvatar name={personaName} hue={personaHue} size={28} />
        </div>
      )}
      <div
        className={`flex max-w-[78%] flex-col ${isUser ? "items-end" : "items-start"}`}
      >
        <div
          className={[
            "rounded-xl px-4 py-2.5 text-[14.5px] leading-relaxed shadow-sm",
            isUser
              ? "bg-[#8B6F47] text-white rounded-br-sm"
              : "bg-white border border-[#E8E0D0] text-[#2C2C2C] rounded-bl-sm",
          ].join(" ")}
        >
          <ReactMarkdown
            components={{
              p: ({ children }) => (
                <p className="whitespace-pre-wrap m-0" style={{ textWrap: "pretty" }}>
                  {children}
                  {isStreaming && <BlinkingCursor />}
                </p>
              ),
              em: ({ children }) => (
                <em className={isUser ? "opacity-80" : "text-[#6B6B6B]"}>{children}</em>
              ),
            }}
          >
            {message.content}
          </ReactMarkdown>
        </div>
      </div>
    </div>
  );
}

function IconButton({
  label,
  onClick,
  children,
}: {
  label: string;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      onClick={onClick}
      className="grid h-8 w-8 place-items-center rounded-md text-[#6B6B6B] transition hover:bg-white hover:text-[#2C2C2C] focus:outline-none focus-visible:ring-2 focus-visible:ring-[#8B6F47]/40"
    >
      {children}
    </button>
  );
}

function SummaryButton({ onClick }: { onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="inline-flex items-center gap-1.5 rounded-md border border-[#E8E0D0] bg-white/80 px-2.5 py-1 text-[12px] font-medium text-[#2C2C2C] backdrop-blur transition hover:border-[#8B6F47] hover:text-[#8B6F47] focus:outline-none focus-visible:ring-2 focus-visible:ring-[#8B6F47]/40"
    >
      <svg
        width="13"
        height="13"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.8"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <rect x="8" y="2" width="8" height="4" rx="1"></rect>
        <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"></path>
        <path d="M9 12h6"></path>
        <path d="M9 16h6"></path>
      </svg>
      Summary
    </button>
  );
}

function ComposerSendButton({
  disabled,
  onClick,
}: {
  disabled: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      aria-label="Send message"
      className="grid h-10 w-10 shrink-0 place-items-center rounded-full bg-[#8B6F47] text-white shadow-sm transition hover:bg-[#7a6040] disabled:cursor-not-allowed disabled:bg-[#C9BCA6] disabled:shadow-none focus:outline-none focus-visible:ring-2 focus-visible:ring-[#8B6F47]/40"
    >
      <svg
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2.2"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <path d="M5 12h14"></path>
        <path d="m13 5 7 7-7 7"></path>
      </svg>
    </button>
  );
}

export default function ChatPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [persona, setPersona] = useState<Persona | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [input, setInput] = useState("");
  const [portraitCompact, setPortraitCompact] = useState(false);
  const [summaryOpen, setSummaryOpen] = useState(false);
  const [initialMessages, setInitialMessages] = useState<Message[]>([]);
  const scrollRef = useRef<HTMLDivElement>(null);

  const { messages, streamingText, isStreaming, sendMessage } =
    useStreamingMessage(id || "", initialMessages);

  useEffect(() => {
    if (!id) {
      setError("Missing conversation ID");
      setLoading(false);
      return;
    }

    getConversation(id)
      .then((conv) => {
        setInitialMessages(conv.messages);
        return getPersonaById(conv.persona_id);
      })
      .then((p) => {
        setPersona(p);
        setLoading(false);
      })
      .catch((err) => {
        setError(err instanceof Error ? err.message : "Failed to load conversation");
        setLoading(false);
      });
  }, [id]);

  useEffect(() => {
    const el = scrollRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [messages, streamingText]);

  if (loading) {
    return (
      <div className="flex h-screen items-center justify-center bg-[#F5F0E8]">
        <div className="text-[#6B6B6B]">Loading conversation…</div>
      </div>
    );
  }

  if (error || !persona) {
    return (
      <div className="flex h-screen flex-col items-center justify-center bg-[#F5F0E8] gap-4">
        <div className="text-[#8B6F47] font-semibold">{error || "Conversation not found"}</div>
        <button
          onClick={() => navigate("/")}
          className="rounded-lg bg-[#8B6F47] px-4 py-2 text-white text-[14px]"
        >
          Back to personas
        </button>
      </div>
    );
  }

  const personaHue = nameToHue(persona.name);
  const backgroundStyle = persona.background_url
    ? { backgroundImage: `url(${persona.background_url})` }
    : {
        background: `radial-gradient(120% 80% at 20% 0%, oklch(0.78 0.05 80 / 0.55), transparent 60%),
          radial-gradient(100% 70% at 90% 30%, oklch(0.62 0.06 150 / 0.45), transparent 65%),
          radial-gradient(120% 90% at 50% 110%, oklch(0.45 0.05 60 / 0.55), transparent 70%),
          linear-gradient(180deg, #F5F0E8 0%, #ECE3D2 100%)`,
      };

  const handleSend = () => {
    const trimmed = input.trim();
    if (!trimmed || isStreaming) return;
    sendMessage(trimmed);
    setInput("");
  };

  return (
    <div
      className="relative flex h-screen w-full flex-col overflow-hidden"
      style={{
        fontFamily:
          'Inter, ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, sans-serif',
        color: "#2C2C2C",
      }}
    >
      <style>{`@keyframes animus-blink { 50% { opacity: 0; } }`}</style>

      <div
        className="pointer-events-none absolute inset-0"
        style={backgroundStyle}
        aria-hidden="true"
      ></div>
      <div
        className="pointer-events-none absolute inset-0"
        style={{ backgroundColor: "rgba(245, 240, 232, 0.78)" }}
        aria-hidden="true"
      ></div>
      <div
        className="pointer-events-none absolute inset-0 opacity-[0.05] mix-blend-multiply"
        style={{
          backgroundImage:
            "repeating-radial-gradient(circle at 0 0, rgba(0,0,0,0.6) 0 1px, transparent 1px 3px)",
        }}
        aria-hidden="true"
      ></div>

      <header className="relative z-10 border-b border-[#E8E0D0] bg-[#F5F0E8]/85 backdrop-blur">
        <div className="mx-auto flex h-11 w-full max-w-6xl items-center gap-3 px-3 sm:px-4">
          <IconButton label="Back to personas" onClick={() => navigate("/")}>
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="m15 18-6-6 6-6"></path>
            </svg>
          </IconButton>
          <div className="flex min-w-0 flex-1 items-center gap-2 text-[13px]">
            <span className="truncate font-semibold text-[#2C2C2C]">
              {persona.name}
            </span>
            <span className="text-[#C9BCA6]">·</span>
            <span className="truncate text-[#6B6B6B]">{persona.description}</span>
          </div>
          <SummaryButton onClick={() => setSummaryOpen(true)} />
        </div>
      </header>

      <div className="relative z-0 mx-auto flex w-full max-w-6xl flex-1 overflow-hidden px-3 sm:px-4">
        <aside
          className={[
            "relative hidden shrink-0 transition-[width] duration-300 ease-out md:block",
            portraitCompact ? "w-[88px]" : "w-[320px] lg:w-[360px]",
          ].join(" ")}
          aria-label="Persona portrait"
        >
          <div className="sticky top-0 flex flex-col gap-3 py-5 pr-4">
            {portraitCompact ? (
              <PortraitSquare
                name={persona.name}
                hue={personaHue}
                className="w-full"
              />
            ) : (
              <>
                <PortraitSquare
                  name={persona.name}
                  hue={personaHue}
                  className="w-full"
                />
                <div className="rounded-xl border border-[#E8E0D0] bg-white/70 p-3 backdrop-blur">
                  <div className="text-[11px] font-medium uppercase tracking-wider text-[#8B6F47]">
                    Persona
                  </div>
                  <div className="mt-0.5 text-[15px] font-semibold text-[#2C2C2C]">
                    {persona.name}
                  </div>
                  <p
                    className="mt-1.5 text-[12.5px] leading-relaxed text-[#6B6B6B]"
                    style={{ textWrap: "pretty" }}
                  >
                    {persona.description}
                  </p>
                </div>
              </>
            )}

            <button
              type="button"
              onClick={() => setPortraitCompact((v) => !v)}
              className="inline-flex items-center justify-center gap-1.5 self-start rounded-md border border-[#E8E0D0] bg-white/80 px-2 py-1 text-[11px] font-medium text-[#6B6B6B] backdrop-blur transition hover:border-[#8B6F47] hover:text-[#8B6F47]"
              aria-label={
                portraitCompact ? "Expand portrait" : "Compact portrait"
              }
            >
              <svg
                width="11"
                height="11"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2.2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                {portraitCompact ? (
                  <path d="m9 18 6-6-6-6"></path>
                ) : (
                  <path d="m15 18-6-6 6-6"></path>
                )}
              </svg>
              {portraitCompact ? "Expand" : "Compact"}
            </button>
          </div>
        </aside>

        <div className="flex min-w-0 flex-1 flex-col">
          <div
            ref={scrollRef}
            className="flex-1 overflow-y-auto"
            style={{ scrollbarGutter: "stable" }}
          >
            <div className="flex w-full flex-col gap-3 py-5">
              <div className="flex items-center justify-center">
                <span className="rounded-full border border-[#E8E0D0] bg-white/70 px-3 py-0.5 text-[11px] uppercase tracking-wider text-[#6B6B6B] backdrop-blur">
                  Today
                </span>
              </div>
              {messages.map((m) => (
                <MessageRow
                  key={m.id}
                  message={m}
                  isStreaming={false}
                  personaHue={personaHue}
                  personaName={persona.name}
                />
              ))}
              {streamingText && (
                <MessageRow
                  message={{
                    id: "streaming",
                    role: "assistant",
                    content: streamingText,
                  }}
                  isStreaming={true}
                  personaHue={personaHue}
                  personaName={persona.name}
                />
              )}
              <div className="h-2"></div>
            </div>
          </div>
        </div>
      </div>

      <footer className="relative z-10 border-t border-[#E8E0D0] bg-[#F5F0E8]/85 backdrop-blur">
        <div className="mx-auto flex w-full max-w-6xl items-center gap-2 px-3 py-3 sm:px-4">
          <div className="flex flex-1 items-center gap-2 rounded-full border border-[#E8E0D0] bg-white px-4 py-2 shadow-sm focus-within:border-[#8B6F47] focus-within:ring-2 focus-within:ring-[#8B6F47]/15">
            <button
              type="button"
              aria-label="Add attachment"
              className="grid h-7 w-7 shrink-0 place-items-center rounded-full text-[#6B6B6B] transition hover:bg-[#F5F0E8] hover:text-[#2C2C2C]"
            >
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <path d="M12 5v14"></path>
                <path d="M5 12h14"></path>
              </svg>
            </button>
            <input
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !e.shiftKey) {
                  e.preventDefault();
                  handleSend();
                }
              }}
              placeholder={`Message ${persona.name.split(" ")[0]}…`}
              className="min-w-0 flex-1 bg-transparent text-[14.5px] text-[#2C2C2C] placeholder:text-[#6B6B6B] focus:outline-none"
            />
            <span className="hidden text-[11px] text-[#6B6B6B] sm:inline">
              <kbd className="rounded border border-[#E8E0D0] bg-[#F5F0E8] px-1 py-0.5 font-mono text-[10px]">
                ↵
              </kbd>
            </span>
          </div>
          <ComposerSendButton disabled={!input.trim() || isStreaming} onClick={handleSend} />
        </div>
      </footer>

      <SummaryDrawer
        open={summaryOpen}
        onClose={() => setSummaryOpen(false)}
        conversationId={id || ""}
        personaName={persona.name}
      />
    </div>
  );
}
