import { useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { createPersona } from "../lib/api";
import type { ContentRating } from "../types/api";

const GLOBAL_DEFAULT_MODEL = "llama3.1:8b";

const inputClass =
  "w-full rounded-lg border border-[#E8E0D0] bg-[#FAFAF6] px-3 py-2 text-[14px] text-[#2C2C2C] placeholder:text-[#6B6B6B]/70 transition focus:border-[#8B6F47] focus:bg-white focus:outline-none focus:ring-2 focus:ring-[#8B6F47]/15";

interface SectionCardProps {
  step: number;
  title: string;
  hint?: string;
  children: React.ReactNode;
}

function SectionCard({ step, title, hint, children }: SectionCardProps) {
  return (
    <section className="rounded-xl border border-[#E8E0D0] bg-white p-6 shadow-sm">
      <header className="mb-5 flex items-baseline gap-2">
        <span className="font-mono text-[11px] tracking-wider text-[#8B6F47]">
          {String(step).padStart(2, "0")}
        </span>
        <h2 className="text-[12px] font-semibold uppercase tracking-[0.14em] text-[#8B6F47]">
          {title}
        </h2>
        {hint && (
          <span className="ml-auto text-[11.5px] text-[#6B6B6B]">{hint}</span>
        )}
      </header>
      <div className="flex flex-col gap-5">{children}</div>
    </section>
  );
}

interface FieldLabelProps {
  label: string;
  required?: boolean;
  htmlFor?: string;
  hint?: React.ReactNode;
}

function FieldLabel({ label, required, htmlFor, hint }: FieldLabelProps) {
  return (
    <div className="mb-1.5 flex items-baseline justify-between gap-3">
      <label
        htmlFor={htmlFor}
        className="text-[13px] font-medium text-[#2C2C2C]"
      >
        {label}
        {required && <span className="ml-1 text-[#B05546]">*</span>}
      </label>
      {hint && <span className="text-[11.5px] text-[#6B6B6B]">{hint}</span>}
    </div>
  );
}

function readAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}

interface AvatarDropzoneProps {
  value: string | undefined;
  onChange: (v: string | undefined) => void;
}

function AvatarDropzone({ value, onChange }: AvatarDropzoneProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [over, setOver] = useState(false);

  const handleFiles = async (files: FileList | null) => {
    if (!files?.[0]) return;
    const url = await readAsDataUrl(files[0]);
    onChange(url);
  };

  return (
    <div className="flex items-center gap-5">
      <button
        type="button"
        onClick={() => inputRef.current?.click()}
        onDragOver={(e) => { e.preventDefault(); setOver(true); }}
        onDragLeave={() => setOver(false)}
        onDrop={(e) => { e.preventDefault(); setOver(false); void handleFiles(e.dataTransfer.files); }}
        className={[
          "group relative grid h-[120px] w-[120px] shrink-0 place-items-center overflow-hidden rounded-full border-2 border-dashed transition",
          over
            ? "border-[#8B6F47] bg-[#F5F0E8]"
            : "border-[#C9BCA6] bg-[#FAFAF6] hover:border-[#8B6F47] hover:bg-[#F5F0E8]",
        ].join(" ")}
        aria-label="Upload avatar image"
      >
        {value ? (
          <>
            <img src={value} alt="Avatar preview" className="h-full w-full object-cover" />
            <span className="absolute inset-0 grid place-items-center bg-black/40 text-[12px] font-medium text-white opacity-0 transition group-hover:opacity-100">
              Replace
            </span>
          </>
        ) : (
          <div className="flex flex-col items-center gap-1.5 text-center text-[#6B6B6B]">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
              <polyline points="17 8 12 3 7 8" />
              <line x1="12" y1="3" x2="12" y2="15" />
            </svg>
            <span className="text-[11px] font-medium">Click or drag</span>
          </div>
        )}
      </button>

      <div className="min-w-0">
        <FieldLabel label="Avatar" hint="Square image, 80–512px" />
        <p className="text-[12.5px] leading-relaxed text-[#6B6B6B]">
          Shown on the persona card and in chat. PNG or JPG.
        </p>
        {value && (
          <button
            type="button"
            onClick={() => onChange(undefined)}
            className="mt-2 text-[12px] font-medium text-[#8B6F47] underline-offset-2 hover:underline"
          >
            Remove
          </button>
        )}
      </div>

      <input
        ref={inputRef}
        type="file"
        accept="image/*"
        className="hidden"
        onChange={(e) => void handleFiles(e.target.files)}
      />
    </div>
  );
}

interface BackgroundDropzoneProps {
  value: string | undefined;
  onChange: (v: string | undefined) => void;
}

function BackgroundDropzone({ value, onChange }: BackgroundDropzoneProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [over, setOver] = useState(false);

  const handleFiles = async (files: FileList | null) => {
    if (!files?.[0]) return;
    const url = await readAsDataUrl(files[0]);
    onChange(url);
  };

  return (
    <div>
      <FieldLabel label="Chat background" hint="Optional · 16:9 recommended" />
      <button
        type="button"
        onClick={() => inputRef.current?.click()}
        onDragOver={(e) => { e.preventDefault(); setOver(true); }}
        onDragLeave={() => setOver(false)}
        onDrop={(e) => { e.preventDefault(); setOver(false); void handleFiles(e.dataTransfer.files); }}
        className={[
          "group relative grid w-full place-items-center overflow-hidden rounded-xl border-2 border-dashed transition",
          over
            ? "border-[#8B6F47] bg-[#F5F0E8]"
            : "border-[#C9BCA6] bg-[#FAFAF6] hover:border-[#8B6F47] hover:bg-[#F5F0E8]",
        ].join(" ")}
        style={{ aspectRatio: "16 / 9" }}
        aria-label="Upload chat background"
      >
        {value ? (
          <>
            <img src={value} alt="Background preview" className="h-full w-full object-cover" />
            <span className="absolute inset-0 grid place-items-center bg-black/40 text-[13px] font-medium text-white opacity-0 transition group-hover:opacity-100">
              Replace background
            </span>
          </>
        ) : (
          <div className="flex flex-col items-center gap-2 text-[#6B6B6B]">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
              <rect x="3" y="3" width="18" height="18" rx="2" />
              <circle cx="9" cy="9" r="2" />
              <path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21" />
            </svg>
            <span className="text-[12.5px] font-medium">Click or drag image</span>
            <span className="text-[11.5px]">Chat background image (optional)</span>
          </div>
        )}
      </button>

      {value && (
        <button
          type="button"
          onClick={() => onChange(undefined)}
          className="mt-2 text-[12px] font-medium text-[#8B6F47] underline-offset-2 hover:underline"
        >
          Remove background
        </button>
      )}

      <input
        ref={inputRef}
        type="file"
        accept="image/*"
        className="hidden"
        onChange={(e) => void handleFiles(e.target.files)}
      />
    </div>
  );
}

const ratingPalette: Record<
  string,
  { active: string; idle: string; dot: string }
> = {
  PG: {
    active: "bg-emerald-50 text-emerald-700 border-emerald-500/50 ring-1 ring-emerald-500/20",
    idle: "border-[#E8E0D0] text-[#6B6B6B] hover:border-emerald-500/40 hover:text-emerald-700",
    dot: "bg-emerald-500",
  },
  Mature: {
    active: "bg-amber-50 text-amber-700 border-amber-500/50 ring-1 ring-amber-500/20",
    idle: "border-[#E8E0D0] text-[#6B6B6B] hover:border-amber-500/40 hover:text-amber-700",
    dot: "bg-amber-500",
  },
  NSFW: {
    active: "bg-rose-50 text-rose-700 border-rose-500/50 ring-1 ring-rose-500/20",
    idle: "border-[#E8E0D0] text-[#6B6B6B] hover:border-rose-500/40 hover:text-rose-700",
    dot: "bg-rose-500",
  },
};

const RATING_LABELS: Record<ContentRating, string> = {
  pg: "PG",
  mature: "Mature",
  nsfw: "NSFW",
};

interface RatingSelectorProps {
  value: ContentRating;
  onChange: (r: ContentRating) => void;
}

function RatingSelector({ value, onChange }: RatingSelectorProps) {
  const ratings: ContentRating[] = ["pg", "mature", "nsfw"];
  return (
    <div>
      <FieldLabel label="Content rating" required hint="Affects defaults & filtering" />
      <div className="grid grid-cols-3 gap-2">
        {ratings.map((r) => {
          const active = value === r;
          const label = RATING_LABELS[r];
          const p = ratingPalette[label];
          return (
            <button
              key={r}
              type="button"
              onClick={() => onChange(r)}
              aria-pressed={active}
              className={[
                "inline-flex items-center justify-center gap-2 rounded-lg border px-3 py-2.5 text-[13px] font-medium transition",
                active ? p.active : `bg-white ${p.idle}`,
              ].join(" ")}
            >
              <span className={`h-1.5 w-1.5 rounded-full ${p.dot}`} />
              {label}
            </button>
          );
        })}
      </div>
    </div>
  );
}

interface DraftState {
  name: string;
  rating: ContentRating;
  description: string;
  personality: string;
  scenario: string;
  firstMessage: string;
  messageExample: string;
  useCustomModel: boolean;
  customModel: string;
  avatarDataUrl: string | undefined;
  bgDataUrl: string | undefined;
}

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
          <SectionCard step={1} title="Identity">
            <AvatarDropzone
              value={draft.avatarDataUrl}
              onChange={(v) => update("avatarDataUrl", v)}
            />
            <BackgroundDropzone
              value={draft.bgDataUrl}
              onChange={(v) => update("bgDataUrl", v)}
            />
            <div>
              <FieldLabel label="Name" required htmlFor="persona-name" />
              <input
                id="persona-name"
                value={draft.name}
                onChange={(e) => update("name", e.target.value)}
                placeholder="e.g. Captain Elara Vance"
                className={inputClass}
              />
            </div>
            <RatingSelector
              value={draft.rating}
              onChange={(r) => update("rating", r)}
            />
          </SectionCard>

          <SectionCard step={2} title="Personality">
            <div>
              <FieldLabel label="Description" htmlFor="persona-desc" hint="Who is this character?" />
              <textarea
                id="persona-desc"
                rows={4}
                value={draft.description}
                onChange={(e) => update("description", e.target.value)}
                placeholder="A stoic deep-space salvage captain who has seen one too many ghost stations…"
                className={`${inputClass} resize-y leading-relaxed`}
              />
            </div>
            <div>
              <FieldLabel label="Personality" htmlFor="persona-traits" hint="Key traits, speaking style" />
              <textarea
                id="persona-traits"
                rows={3}
                value={draft.personality}
                onChange={(e) => update("personality", e.target.value)}
                placeholder="Clipped sentences. Distrustful but loyal. Drinks coffee black. Uses spacer slang."
                className={`${inputClass} resize-y leading-relaxed`}
              />
            </div>
            <div>
              <FieldLabel label="Scenario" htmlFor="persona-scenario" hint="Setting and context" />
              <textarea
                id="persona-scenario"
                rows={3}
                value={draft.scenario}
                onChange={(e) => update("scenario", e.target.value)}
                placeholder="Aboard the salvage hauler 'Astrid's Verdict', three days out from the Hyacinth drift…"
                className={`${inputClass} resize-y leading-relaxed`}
              />
            </div>
          </SectionCard>

          <SectionCard step={3} title="Dialogue">
            <div>
              <FieldLabel
                label="First message"
                htmlFor="persona-first"
                hint={
                  <span>
                    Use{" "}
                    <code className="rounded bg-[#F5F0E8] px-1 py-0.5 font-mono text-[11px] text-[#8B6F47]">
                      *asterisks*
                    </code>{" "}
                    for actions
                  </span>
                }
              />
              <textarea
                id="persona-first"
                rows={4}
                value={draft.firstMessage}
                onChange={(e) => update("firstMessage", e.target.value)}
                placeholder={`*Elara doesn't look up from the console.* You the new tech? Strap in. We jump in three.`}
                className={`${inputClass} resize-y font-mono text-[13.5px] leading-relaxed`}
                spellCheck={false}
              />
              <p className="mt-1.5 text-[11.5px] text-[#6B6B6B]">
                The opening message the character sends when a chat starts.
              </p>
            </div>
            <div>
              <FieldLabel
                label="Message examples"
                htmlFor="persona-examples"
                hint="Optional · sample exchanges"
              />
              <textarea
                id="persona-examples"
                rows={4}
                value={draft.messageExample}
                onChange={(e) => update("messageExample", e.target.value)}
                placeholder={`<START>\n{{user}}: You okay?\n{{char}}: *doesn't answer immediately* Define "okay".`}
                className={`${inputClass} resize-y font-mono text-[13.5px] leading-relaxed`}
                spellCheck={false}
              />
              <p className="mt-1.5 text-[11.5px] text-[#6B6B6B]">
                Sample exchanges that illustrate the character's voice and style.
              </p>
            </div>
          </SectionCard>

          <SectionCard step={4} title="Model override" hint="Optional">
            <div className="flex items-start justify-between gap-4">
              <div className="min-w-0">
                <div className="text-[13px] font-medium text-[#2C2C2C]">Use custom model</div>
                <p className="mt-0.5 text-[12.5px] leading-relaxed text-[#6B6B6B]">
                  Override the global default just for this persona — useful for
                  characters that benefit from a different size or fine-tune.
                </p>
              </div>
              <button
                type="button"
                role="switch"
                aria-checked={draft.useCustomModel}
                onClick={() => update("useCustomModel", !draft.useCustomModel)}
                className={[
                  "relative h-6 w-11 shrink-0 rounded-full transition",
                  draft.useCustomModel ? "bg-[#8B6F47]" : "bg-[#D9CFB9]",
                ].join(" ")}
              >
                <span
                  className={[
                    "absolute top-0.5 h-5 w-5 rounded-full bg-white shadow transition-transform",
                    draft.useCustomModel ? "translate-x-5" : "translate-x-0.5",
                  ].join(" ")}
                />
              </button>
            </div>

            <div
              className={[
                "grid transition-[grid-template-rows,opacity] duration-300",
                draft.useCustomModel ? "grid-rows-[1fr] opacity-100" : "grid-rows-[0fr] opacity-0",
              ].join(" ")}
              aria-hidden={!draft.useCustomModel}
            >
              <div className="overflow-hidden">
                <FieldLabel label="Model" htmlFor="persona-model" />
                <input
                  id="persona-model"
                  value={draft.customModel}
                  onChange={(e) => update("customModel", e.target.value)}
                  placeholder={GLOBAL_DEFAULT_MODEL}
                  disabled={!draft.useCustomModel}
                  className={`${inputClass} font-mono text-[13.5px]`}
                />
                <p className="mt-1.5 text-[11.5px] text-[#6B6B6B]">
                  Leave empty to use global default (
                  <span className="font-mono text-[#2C2C2C]">{GLOBAL_DEFAULT_MODEL}</span>
                  ).
                </p>
              </div>
            </div>
          </SectionCard>
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
