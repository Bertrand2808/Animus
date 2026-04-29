import type { ContentRating } from "../../types/api";
import { AvatarDropzone } from "./AvatarDropzone";
import { BackgroundDropzone } from "./BackgroundDropzone";
import { FieldLabel, inputClass } from "./FieldLabel";
import { RatingSelector } from "./RatingSelector";
import { SectionCard } from "./SectionCard";

export const GLOBAL_DEFAULT_MODEL = "llama3.1:8b";

export interface DraftState {
  name: string;
  rating: ContentRating;
  description: string;
  personality: string;
  scenario: string;
  firstMessage: string;
  messageExample: string;
  modelInstructions: string;
  appearance: string;
  speechStyle: string;
  characterGoals: string;
  postHistoryInstructions: string;
  responseLengthLimit: number;
  temperature: number;
  repeatPenalty: number;
  instructionTemplate: string;
  useCustomModel: boolean;
  customModel: string;
  avatarDataUrl: string | undefined;
  bgDataUrl: string | undefined;
}

export interface PersonaFormFieldsProps {
  draft: DraftState;
  onChange: <K extends keyof DraftState>(key: K, val: DraftState[K]) => void;
}

function numericValue(value: number, fallback: number) {
  return Number.isFinite(value) ? value : fallback;
}

export function PersonaFormFields({ draft, onChange }: PersonaFormFieldsProps) {
  return (
    <>
      <SectionCard step={1} title="Identity">
        <AvatarDropzone
          value={draft.avatarDataUrl}
          onChange={(v) => onChange("avatarDataUrl", v)}
        />
        <BackgroundDropzone
          value={draft.bgDataUrl}
          onChange={(v) => onChange("bgDataUrl", v)}
        />
        <div>
          <FieldLabel label="Name" required htmlFor="persona-name" />
          <input
            id="persona-name"
            value={draft.name}
            onChange={(e) => onChange("name", e.target.value)}
            placeholder="e.g. Captain Elara Vance"
            className={inputClass}
          />
        </div>
        <RatingSelector
          value={draft.rating}
          onChange={(r) => onChange("rating", r)}
        />
      </SectionCard>

      <SectionCard step={2} title="Personality">
        <div>
          <FieldLabel label="Description" htmlFor="persona-desc" hint="Who is this character?" />
          <textarea
            id="persona-desc"
            rows={4}
            value={draft.description}
            onChange={(e) => onChange("description", e.target.value)}
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
            onChange={(e) => onChange("personality", e.target.value)}
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
            onChange={(e) => onChange("scenario", e.target.value)}
            placeholder="Aboard the salvage hauler 'Astrid's Verdict', three days out from the Hyacinth drift…"
            className={`${inputClass} resize-y leading-relaxed`}
          />
        </div>
      </SectionCard>

      <SectionCard step={3} title="Character driving">
        <div>
          <FieldLabel label="Model instructions" htmlFor="persona-model-instructions" />
          <textarea
            id="persona-model-instructions"
            rows={4}
            value={draft.modelInstructions}
            onChange={(e) => onChange("modelInstructions", e.target.value)}
            placeholder="Behavioral rules the model should follow when playing this persona."
            className={`${inputClass} resize-y leading-relaxed`}
          />
        </div>
        <div>
          <FieldLabel label="Appearance" htmlFor="persona-appearance" />
          <textarea
            id="persona-appearance"
            rows={3}
            value={draft.appearance}
            onChange={(e) => onChange("appearance", e.target.value)}
            placeholder="Visual details, clothing, posture, expressions, or other physical cues."
            className={`${inputClass} resize-y leading-relaxed`}
          />
        </div>
        <div>
          <FieldLabel label="Speech style" htmlFor="persona-speech-style" />
          <textarea
            id="persona-speech-style"
            rows={3}
            value={draft.speechStyle}
            onChange={(e) => onChange("speechStyle", e.target.value)}
            placeholder="Tone, rhythm, vocabulary, punctuation habits, and recurring phrases."
            className={`${inputClass} resize-y leading-relaxed`}
          />
        </div>
        <div>
          <FieldLabel label="Character goals" htmlFor="persona-character-goals" />
          <textarea
            id="persona-character-goals"
            rows={3}
            value={draft.characterGoals}
            onChange={(e) => onChange("characterGoals", e.target.value)}
            placeholder="What the character wants, avoids, protects, or tries to achieve."
            className={`${inputClass} resize-y leading-relaxed`}
          />
        </div>
        <div>
          <FieldLabel label="Post-history instructions" htmlFor="persona-post-history" />
          <textarea
            id="persona-post-history"
            rows={3}
            value={draft.postHistoryInstructions}
            onChange={(e) => onChange("postHistoryInstructions", e.target.value)}
            placeholder="Instructions to apply after conversation history is included."
            className={`${inputClass} resize-y leading-relaxed`}
          />
        </div>
      </SectionCard>

      <SectionCard step={4} title="Dialogue">
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
            onChange={(e) => onChange("firstMessage", e.target.value)}
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
            onChange={(e) => onChange("messageExample", e.target.value)}
            placeholder={`<START>\n{{user}}: You okay?\n{{char}}: *doesn't answer immediately* Define "okay".`}
            className={`${inputClass} resize-y font-mono text-[13.5px] leading-relaxed`}
            spellCheck={false}
          />
          <p className="mt-1.5 text-[11.5px] text-[#6B6B6B]">
            Sample exchanges that illustrate the character's voice and style.
          </p>
        </div>
      </SectionCard>

      <SectionCard step={5} title="Generation settings">
        <div className="grid gap-4 sm:grid-cols-3">
          <div>
            <FieldLabel label="Response limit" htmlFor="persona-response-limit" hint="characters" />
            <input
              id="persona-response-limit"
              type="number"
              min={1}
              step={1}
              value={draft.responseLengthLimit}
              onChange={(e) =>
                onChange("responseLengthLimit", numericValue(e.target.valueAsNumber, 1200))
              }
              className={inputClass}
            />
          </div>
          <div>
            <FieldLabel label="Temperature" htmlFor="persona-temperature" />
            <input
              id="persona-temperature"
              type="number"
              min={0}
              max={2}
              step={0.01}
              value={draft.temperature}
              onChange={(e) =>
                onChange("temperature", numericValue(e.target.valueAsNumber, 0.65))
              }
              className={inputClass}
            />
          </div>
          <div>
            <FieldLabel label="Repeat penalty" htmlFor="persona-repeat-penalty" />
            <input
              id="persona-repeat-penalty"
              type="number"
              min={0}
              step={0.01}
              value={draft.repeatPenalty}
              onChange={(e) =>
                onChange("repeatPenalty", numericValue(e.target.valueAsNumber, 1.12))
              }
              className={inputClass}
            />
          </div>
        </div>
        <div>
          <FieldLabel label="Instruction template" htmlFor="persona-instruction-template" />
          <input
            id="persona-instruction-template"
            value={draft.instructionTemplate}
            onChange={(e) => onChange("instructionTemplate", e.target.value)}
            placeholder="default"
            className={`${inputClass} font-mono text-[13.5px]`}
          />
        </div>
      </SectionCard>

      <SectionCard step={6} title="Model override" hint="Optional">
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
            onClick={() => onChange("useCustomModel", !draft.useCustomModel)}
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
              onChange={(e) => onChange("customModel", e.target.value)}
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
    </>
  );
}
