import type {
  ApiSummary,
  ApiSummaryResponse,
  ConversationDetail,
  ConversationResponse,
  ConversationSummary,
  OllamaStatus,
  Persona,
} from "../types/api";

async function request<T>(url: string, init?: RequestInit): Promise<T> {
  const res = await fetch(url, init);
  if (!res.ok) {
    const text = await res.text().catch(() => res.statusText);
    throw new Error(`${res.status}: ${text}`);
  }
  return res.json() as Promise<T>;
}

export function listPersonas(): Promise<Persona[]> {
  return request<Persona[]>("/api/personas");
}

export function importPersona(json: string): Promise<Persona> {
  return request<Persona>("/api/personas/import", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: json,
  });
}

export function getOllamaStatus(): Promise<OllamaStatus> {
  return request<OllamaStatus>("/api/ollama/status");
}

export async function getLatestConversation(
  personaId: string,
): Promise<ConversationSummary | null> {
  const data = await request<{ conversation: ConversationSummary | null }>(
    `/api/conversations?persona_id=${personaId}`,
  );
  return data.conversation;
}

export function createConversation(
  personaId: string,
): Promise<ConversationResponse> {
  return request<ConversationResponse>("/api/conversations", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ persona_id: personaId }),
  });
}

export function getConversation(id: string): Promise<ConversationDetail> {
  return request<ConversationDetail>(`/api/conversations/${id}`);
}

export function getPersonaById(id: string): Promise<Persona> {
  return request<Persona>(`/api/personas/${id}`);
}

export async function getSummary(
  conversationId: string,
): Promise<ApiSummary | null> {
  const data = await request<ApiSummaryResponse>(
    `/api/conversations/${conversationId}/summary`,
  );
  if (!data.content || !data.message_range_start || !data.message_range_end || data.created_at === null) {
    return null;
  }
  return {
    content: data.content,
    message_range_start: data.message_range_start,
    message_range_end: data.message_range_end,
    created_at: data.created_at * 1000,
  };
}

export function streamMessage(
  conversationId: string,
  content: string,
): Promise<Response> {
  return fetch(`/api/conversations/${conversationId}/messages`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Accept: "text/event-stream",
    },
    body: JSON.stringify({ content }),
  });
}
