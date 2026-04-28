import type { ContentRating, Persona } from "../types/api";

interface Props {
  persona: Persona;
  onClick: () => void;
  onEdit: () => void;
}

const ratingLabel: Record<ContentRating, string> = {
  pg: "PG",
  mature: "Mature",
  nsfw: "NSFW",
};

const ratingStyle: Record<ContentRating, string> = {
  pg: "bg-emerald-50 text-emerald-700 ring-emerald-600/20",
  mature: "bg-amber-50 text-amber-700 ring-amber-600/20",
  nsfw: "bg-rose-50 text-rose-700 ring-rose-600/20",
};

const ratingDot: Record<ContentRating, string> = {
  pg: "bg-emerald-500",
  mature: "bg-amber-500",
  nsfw: "bg-rose-500",
};

function AvatarPlaceholder({ name }: { name: string }) {
  const initials = name
    .split(" ")
    .filter((w) => /^[A-Z]/.test(w))
    .slice(0, 2)
    .map((w) => w[0])
    .join("");
  return (
    <div
      className="relative h-20 w-20 shrink-0 overflow-hidden rounded-full bg-[#E8E0D0] ring-1 ring-[#E8E0D0]"
      aria-hidden
    >
      <div className="absolute inset-0 grid place-items-center font-mono text-[15px] tracking-wider text-[#2C2C2C]/60">
        {initials || "··"}
      </div>
    </div>
  );
}

export function PersonaCard({ persona, onClick, onEdit }: Props) {
  return (
    <div className="group flex w-full flex-col rounded-xl border border-[#E8E0D0] bg-white shadow-sm transition-all duration-200 hover:-translate-y-0.5 hover:border-[#8B6F47] hover:shadow-md">
      {/* Main clickable area */}
      <button
        type="button"
        onClick={onClick}
        className="flex-1 p-5 text-left focus:outline-none focus-visible:ring-2 focus-visible:ring-[#8B6F47]/40 rounded-t-xl"
      >
        <div className="flex items-start gap-4">
          {persona.avatar_url ? (
            <img
              src={persona.avatar_url}
              alt={persona.name}
              className="h-20 w-20 shrink-0 rounded-full object-cover ring-1 ring-[#E8E0D0]"
            />
          ) : (
            <AvatarPlaceholder name={persona.name} />
          )}
          <div className="min-w-0 flex-1">
            <div className="flex items-start justify-between gap-2">
              <h3 className="truncate text-[18px] font-semibold leading-tight text-[#2C2C2C]">
                {persona.name}
              </h3>
              <span
                className={`inline-flex shrink-0 items-center gap-1.5 rounded-full px-2 py-0.5 text-[11px] font-medium ring-1 ring-inset ${ratingStyle[persona.content_rating]}`}
              >
                <span className={`h-1.5 w-1.5 rounded-full ${ratingDot[persona.content_rating]}`} />
                {ratingLabel[persona.content_rating]}
              </span>
            </div>
            <p
              className="mt-2 text-[14px] leading-relaxed text-[#6B6B6B]"
              style={{
                display: "-webkit-box",
                WebkitLineClamp: 2,
                WebkitBoxOrient: "vertical",
                overflow: "hidden",
              }}
            >
              {persona.description || "No description."}
            </p>
          </div>
        </div>
      </button>

      {/* Footer actions */}
      <div className="flex items-center justify-between border-t border-[#E8E0D0]/70 px-5 py-2.5 text-[12px]">
        <button
          type="button"
          onClick={onEdit}
          className="inline-flex items-center gap-1.5 font-medium text-[#6B6B6B] opacity-0 transition-opacity group-hover:opacity-100 hover:text-[#8B6F47] focus:outline-none focus-visible:ring-2 focus-visible:ring-[#8B6F47]/40 rounded px-1 -mx-1"
        >
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
          </svg>
          Edit
        </button>
        <button
          type="button"
          onClick={onClick}
          className="inline-flex items-center gap-1 font-medium text-[#8B6F47] opacity-0 transition-opacity group-hover:opacity-100 focus:outline-none focus-visible:ring-2 focus-visible:ring-[#8B6F47]/40 rounded px-1 -mx-1"
        >
          Open
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M5 12h14" />
            <path d="m12 5 7 7-7 7" />
          </svg>
        </button>
      </div>
    </div>
  );
}
