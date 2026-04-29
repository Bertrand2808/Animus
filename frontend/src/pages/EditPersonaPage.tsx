import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import type { DraftState } from "../components/persona-form";
import {
  GLOBAL_DEFAULT_MODEL,
  PersonaFormFields,
} from "../components/persona-form";
import { getPersonaById, updatePersona } from "../lib/api";
import type { Persona } from "../types/api";

function personaToDraft(p: Persona): DraftState {
  return {
    name: p.name,
    rating: p.content_rating,
    description: p.description,
    personality: p.personality,
    scenario: p.scenario,
    firstMessage: p.first_message,
    messageExample: p.message_example,
    modelInstructions: p.model_instructions,
    appearance: p.appearance,
    speechStyle: p.speech_style,
    characterGoals: p.character_goals,
    postHistoryInstructions: p.post_history_instructions,
    responseLengthLimit: p.response_length_limit,
    temperature: p.temperature,
    repeatPenalty: p.repeat_penalty,
    instructionTemplate: p.instruction_template,
    useCustomModel: p.model !== null,
    customModel: p.model ?? GLOBAL_DEFAULT_MODEL,
    avatarDataUrl: p.avatar_url ?? undefined,
    bgDataUrl: p.background_url ?? undefined,
  };
}

export default function EditPersonaPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const [draft, setDraft] = useState<DraftState | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) return;
    getPersonaById(id)
      .then((p) => setDraft(personaToDraft(p)))
      .catch(() => setLoadError("Failed to load persona."));
  }, [id]);

  const update = <K extends keyof DraftState>(key: K, val: DraftState[K]) =>
    setDraft((d) => (d ? { ...d, [key]: val } : d));

  const canSubmit = draft !== null && draft.name.trim().length > 0 && !submitting;

  const handleSubmit = async () => {
    if (!canSubmit || !id || !draft) return;
    setSubmitting(true);
    setSubmitError(null);
    try {
      await updatePersona(id, {
        name: draft.name.trim(),
        description: draft.description || undefined,
        personality: draft.personality || undefined,
        scenario: draft.scenario || undefined,
        first_message: draft.firstMessage || undefined,
        message_example: draft.messageExample || undefined,
        content_rating: draft.rating,
        model: draft.useCustomModel && draft.customModel ? draft.customModel : null,
        avatar_url: draft.avatarDataUrl ?? null,
        background_url: draft.bgDataUrl ?? null,
        model_instructions: draft.modelInstructions,
        appearance: draft.appearance,
        speech_style: draft.speechStyle,
        character_goals: draft.characterGoals,
        post_history_instructions: draft.postHistoryInstructions,
        response_length_limit: draft.responseLengthLimit,
        temperature: draft.temperature,
        repeat_penalty: draft.repeatPenalty,
        instruction_template: draft.instructionTemplate || "default",
      });
      void navigate("/");
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Unknown error";
      setSubmitError(
        msg.startsWith("409") ? "A persona with this name already exists." : msg,
      );
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
            Edit persona
          </h1>
          <span className="ml-auto text-[11.5px] text-[#6B6B6B]">
            {draft?.name ?? "Loading…"}
          </span>
        </div>
      </header>

      <main className="mx-auto max-w-[680px] px-4 pb-32 pt-8">
        {loadError ? (
          <div className="rounded-lg border border-rose-200 bg-rose-50 px-4 py-3 text-[13.5px] text-rose-700">
            {loadError}
          </div>
        ) : !draft ? (
          <div className="flex flex-col gap-5">
            {[1, 2, 3, 4].map((i) => (
              <div
                key={i}
                className="h-40 animate-pulse rounded-xl border border-[#E8E0D0] bg-white"
              />
            ))}
          </div>
        ) : (
          <>
            <div className="mb-6">
              <h2 className="text-[24px] font-semibold tracking-tight text-[#2C2C2C]">
                Edit persona
              </h2>
              <p className="mt-1.5 text-[14px] leading-relaxed text-[#6B6B6B]" style={{ textWrap: "pretty" }}>
                Changes are saved locally — nothing is uploaded.
              </p>
            </div>

            {submitError && (
              <div className="mb-5 rounded-lg border border-rose-200 bg-rose-50 px-4 py-3 text-[13.5px] text-rose-700">
                {submitError}
              </div>
            )}

            <div className="flex flex-col gap-5">
              <PersonaFormFields draft={draft} onChange={update} />
            </div>
          </>
        )}
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
            {draft && !draft.name.trim() && (
              <span className="text-[11.5px] text-[#6B6B6B]">Name is required</span>
            )}
            <button
              type="button"
              disabled={!canSubmit}
              onClick={() => void handleSubmit()}
              className="inline-flex items-center gap-2 rounded-lg bg-[#8B6F47] px-4 py-2 text-[13px] font-semibold text-white shadow-sm transition hover:bg-[#7a6040] disabled:cursor-not-allowed disabled:bg-[#C9BCA6] disabled:shadow-none focus:outline-none focus-visible:ring-2 focus-visible:ring-[#8B6F47]/40"
            >
              {submitting ? "Saving…" : "Save changes"}
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
