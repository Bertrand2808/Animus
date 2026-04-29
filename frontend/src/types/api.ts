export type ContentRating = "pg" | "mature" | "nsfw";

export interface Persona {
  id: string;
  name: string;
  description: string;
  personality: string;
  scenario: string;
  first_message: string;
  message_example: string;
  avatar_url: string | null;
  background_url: string | null;
  content_rating: ContentRating;
  model: string | null;
  model_instructions: string;
  appearance: string;
  speech_style: string;
  character_goals: string;
  post_history_instructions: string;
  response_length_limit: number;
  temperature: number;
  repeat_penalty: number;
  instruction_template: string;
}

export interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
  token_count?: number;
}

export interface ConversationDetail {
  id: string;
  persona_id: string;
  created_at: number;
  messages: Message[];
}

export interface ConversationSummary {
  id: string;
  persona_id: string;
  created_at: number;
}

export interface ConversationResponse {
  id: string;
  persona_id: string;
  first_message: string;
}

export interface ApiSummaryResponse {
  id: string | null;
  conversation_id: string;
  content: string | null;
  message_range_start: string | null;
  message_range_end: string | null;
  created_at: number | null;
}

export interface ApiSummary {
  content: string;
  message_range_start: string;
  message_range_end: string;
  created_at: number;
}

export interface OllamaStatus {
  online: boolean;
  model: string;
}

export interface UpdatePersonaRequest {
  name: string;
  description?: string;
  personality?: string;
  scenario?: string;
  first_message?: string;
  message_example?: string;
  content_rating?: ContentRating;
  model?: string | null;
  avatar_url?: string | null;
  background_url?: string | null;
  model_instructions?: string;
  appearance?: string;
  speech_style?: string;
  character_goals?: string;
  post_history_instructions?: string;
  response_length_limit?: number;
  temperature?: number;
  repeat_penalty?: number;
  instruction_template?: string;
}

export interface CreatePersonaRequest {
  name: string;
  description?: string;
  personality?: string;
  scenario?: string;
  first_message?: string;
  message_example?: string;
  content_rating?: ContentRating;
  model?: string;
  avatar_url?: string;
  background_url?: string;
  model_instructions?: string;
  appearance?: string;
  speech_style?: string;
  character_goals?: string;
  post_history_instructions?: string;
  response_length_limit?: number;
  temperature?: number;
  repeat_penalty?: number;
  instruction_template?: string;
}
