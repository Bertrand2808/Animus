import type { ResponseStylePreset } from "@/types/api";
export { RESPONSE_STYLE_PRESETS } from "../utils/persona-form-utils";
import { FieldLabel } from "./FieldLabel";

export const STYLE_PRESETS: Record<ResponseStylePreset, string> = {
  balanced: "Balanced",
  creative: "Creative",
  stable: "Stable",
};

export const stylePresetPalette: Record<
  string,
  { active: string; idle: string; dot: string }
> = {
  BALANCED: {
    active:
      "bg-emerald-50 text-emerald-700 border-emerald-500/50 ring-1 ring-emerald-500/20",
    idle: "border-[#E8E0D0] text-[#6B6B6B] hover:border-emerald-500/40 hover:text-emerald-700",
    dot: "bg-emerald-500",
  },
  CREATIVE: {
    active:
      "bg-amber-50 text-amber-700 border-amber-500/50 ring-1 ring-amber-500/20",
    idle: "border-[#E8E0D0] text-[#6B6B6B] hover:border-amber-500/40 hover:text-amber-700",
    dot: "bg-amber-500",
  },
  STABLE: {
    active:
      "bg-rose-50 text-rose-700 border-rose-500/50 ring-1 ring-rose-500/20",
    idle: "border-[#E8E0D0] text-[#6B6B6B] hover:border-rose-500/40 hover:text-rose-700",
    dot: "bg-rose-500",
  },
};

export interface ResponseStylePresetSelectorProps {
  value: ResponseStylePreset;
  onApplyPreset: (preset: ResponseStylePreset) => void;
}

export function ResponseStylePresetSelector({
  value,
  onApplyPreset,
}: ResponseStylePresetSelectorProps) {
  const stylePresets: ResponseStylePreset[] = [
    "balanced",
    "creative",
    "stable",
  ];
  return (
    <div>
      <FieldLabel
        label="Response Style Preset"
        required
        hint="Affects temperature & repeatPenalty"
      />
      <div className="flex flex-wrap gap-2">
        {stylePresets.map((s) => {
          const active = value === s;
          const label = STYLE_PRESETS[s];
          const p = stylePresetPalette[label.toUpperCase()];
          return (
            <button
              key={s}
              type="button"
              onClick={() => onApplyPreset(s)}
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
