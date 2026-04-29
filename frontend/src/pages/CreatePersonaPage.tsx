import { useState } from "react";
import { useNavigate } from "react-router-dom";
import type { DraftState } from "../components/persona-form";
import {
  GLOBAL_DEFAULT_MODEL,
  PersonaFormFields,
} from "../components/persona-form";
import { createPersona } from "../lib/api";

export default function CreatePersonaPage() {
  const navigate = useNavigate();
  const [draft, setDraft] = useState<DraftState>({
    name: "",
    rating: "pg",
    description: "",
    personality: "",
    scenario: "",
    firstMessage: "",
    messageExample: "",
    modelInstructions: "",
    appearance: "",
    speechStyle: "",
    characterGoals: "",
    postHistoryInstructions: "",
    responseLengthLimit: 1200,
    temperature: 0.65,
    repeatPenalty: 1.12,
    instructionTemplate: "default",
    useCustomModel: false,
    customModel: GLOBAL_DEFAULT_MODEL,
    avatarDataUrl: undefined,
    bgDataUrl: undefined,
  });
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const update = <K extends keyof DraftState>(key: K, val: DraftState[K]) =>
    setDraft((d) => ({ ...d, [key]: val }));

  const canSubmit = draft.name.trim().length > 0 && !submitting;

  const handleSubmit = async () => {
    if (!canSubmit) return;
    setSubmitting(true);
    setError(null);
    try {
      await createPersona({
        name: draft.name.trim(),
        description: draft.description || undefined,
        personality: draft.personality || undefined,
        scenario: draft.scenario || undefined,
        first_message: draft.firstMessage || undefined,
        message_example: draft.messageExample || undefined,
        content_rating: draft.rating,
        model: draft.useCustomModel && draft.customModel ? draft.customModel : undefined,
        avatar_url: draft.avatarDataUrl,
        background_url: draft.bgDataUrl,
        model_instructions: draft.modelInstructions || undefined,
        appearance: draft.appearance || undefined,
        speech_style: draft.speechStyle || undefined,
        character_goals: draft.characterGoals || undefined,
        post_history_instructions: draft.postHistoryInstructions || undefined,
        response_length_limit: draft.responseLengthLimit,
        temperature: draft.temperature,
        repeat_penalty: draft.repeatPenalty,
        instruction_template: draft.instructionTemplate || undefined,
      });
      void navigate("/");
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Unknown error";
      setError(msg.startsWith("409") ? "A persona with this name already exists." : msg);
      setSubmitting(false);
    }
  };

  return (
    <div
      className="min-h-screen w-full"
      style={{
        backgroundColor: "#F5F0E8",
        fontFamily: 'Inter, ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, sans-serif',
        color: "#2C2C2C",
      }}
    >
      <header className="sticky top-0 z-20 border-b border-[#E8E0D0] bg-[#F5F0E8]/85 backdrop-blur">
        <div className="mx-auto flex h-12 max-w-[680px] items-center gap-3 px-4">
          <button
            type="button"
            aria-label="Back"
            onClick={() => navigate(-1)}
            className="grid h-8 w-8 place-items-center rounded-md text-[#6B6B6B] transition hover:bg-white hover:text-[#2C2C2C] focus:outline-none focus-visible:ring-2 focus-visible:ring-[#8B6F47]/40"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="m15 18-6-6 6-6" />
            </svg>
          </button>
          <h1 className="text-[14px] font-semibold tracking-tight text-[#2C2C2C]">
            New persona
          </h1>
          <span className="ml-auto text-[11.5px] text-[#6B6B6B]">
            Manual creation
          </span>
        </div>
      </header>

      <main className="mx-auto max-w-[680px] px-4 pb-32 pt-8">
        <div className="mb-6">
          <h2 className="text-[24px] font-semibold tracking-tight text-[#2C2C2C]">
            Create a persona
          </h2>
          <p className="mt-1.5 text-[14px] leading-relaxed text-[#6B6B6B]" style={{ textWrap: "pretty" }}>
            Define who they are, how they speak, and the world they live in.
            Everything below stays on your machine — nothing is uploaded.
          </p>
        </div>

        {error && (
          <div className="mb-5 rounded-lg border border-rose-200 bg-rose-50 px-4 py-3 text-[13.5px] text-rose-700">
            {error}
          </div>
        )}

        <div className="flex flex-col gap-5">
          <PersonaFormFields draft={draft} onChange={update} />
        </div>
      </main>

      <footer className="fixed inset-x-0 bottom-0 z-20 border-t border-[#E8E0D0] bg-[#F5F0E8]/90 backdrop-blur">
        <div className="mx-auto flex max-w-[680px] items-center justify-between gap-3 px-4 py-3">
          <button
            type="button"
            onClick={() => navigate(-1)}
            className="rounded-lg px-3 py-2 text-[13px] font-medium text-[#6B6B6B] transition hover:bg-white hover:text-[#2C2C2C] focus:outline-none focus-visible:ring-2 focus-visible:ring-[#8B6F47]/40"
          >
            Cancel
          </button>
          <div className="flex items-center gap-3">
            {!draft.name.trim() && (
              <span className="text-[11.5px] text-[#6B6B6B]">Name is required</span>
            )}
            <button
              type="button"
              disabled={!canSubmit}
              onClick={() => void handleSubmit()}
              className="inline-flex items-center gap-2 rounded-lg bg-[#8B6F47] px-4 py-2 text-[13px] font-semibold text-white shadow-sm transition hover:bg-[#7a6040] disabled:cursor-not-allowed disabled:bg-[#C9BCA6] disabled:shadow-none focus:outline-none focus-visible:ring-2 focus-visible:ring-[#8B6F47]/40"
            >
              {submitting ? "Creating…" : "Create persona"}
              {!submitting && (
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M5 12h14" />
                  <path d="m12 5 7 7-7 7" />
                </svg>
              )}
            </button>
          </div>
        </div>
      </footer>
    </div>
  );
}
