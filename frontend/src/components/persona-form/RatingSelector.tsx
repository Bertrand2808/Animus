import type { ContentRating } from "../../types/api";
import { FieldLabel } from "./FieldLabel";

export const RATING_LABELS: Record<ContentRating, string> = {
  pg: "PG",
  mature: "Mature",
  nsfw: "NSFW",
};

export const ratingPalette: Record<
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

export interface RatingSelectorProps {
  value: ContentRating;
  onChange: (r: ContentRating) => void;
}

export function RatingSelector({ value, onChange }: RatingSelectorProps) {
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
