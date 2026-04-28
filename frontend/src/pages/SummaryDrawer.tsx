import { useEffect, useState } from "react";
import { getSummary } from "../lib/api";
import type { ApiSummary } from "../types/api";

interface SummaryDrawerProps {
  open: boolean;
  onClose: () => void;
  conversationId: string;
  personaName: string;
}

function relativeTime(timestamp: number): string {
  const diffMs = Date.now() - timestamp;
  const m = Math.round(diffMs / 60000);
  if (m < 1) return "just now";
  if (m < 60) return `${m} minute${m === 1 ? "" : "s"} ago`;
  const h = Math.round(m / 60);
  if (h < 24) return `${h} hour${h === 1 ? "" : "s"} ago`;
  const days = Math.round(h / 24);
  return `${days} day${days === 1 ? "" : "s"} ago`;
}

function CloseButton({ onClick }: { onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-label="Close summary"
      className="grid h-8 w-8 place-items-center rounded-md text-[#6B6B6B] transition hover:bg-[#F5F0E8] hover:text-[#2C2C2C] focus:outline-none focus-visible:ring-2 focus-visible:ring-[#8B6F47]/40"
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
        <path d="M18 6 6 18"></path>
        <path d="m6 6 12 12"></path>
      </svg>
    </button>
  );
}

function MetaRow({
  label,
  value,
}: {
  label: string;
  value: React.ReactNode;
}) {
  return (
    <div className="flex items-baseline justify-between gap-3 text-[12.5px]">
      <span className="text-[#6B6B6B]">{label}</span>
      <span className="text-right font-medium text-[#2C2C2C]">{value}</span>
    </div>
  );
}

function SummaryEmptyState() {
  return (
    <div className="flex flex-1 flex-col items-center justify-center px-6 text-center">
      <div
        className="mb-4 grid h-14 w-14 place-items-center rounded-full border border-[#E8E0D0] bg-white text-[#8B6F47] shadow-sm"
        aria-hidden="true"
      >
        <svg
          width="22"
          height="22"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.6"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <rect x="8" y="2" width="8" height="4" rx="1"></rect>
          <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"></path>
          <path d="M9 12h6"></path>
          <path d="M9 16h4"></path>
        </svg>
      </div>
      <h3 className="text-[15px] font-semibold text-[#2C2C2C]">
        No summary yet
      </h3>
      <p className="mt-1.5 max-w-[260px] text-[13px] leading-relaxed text-[#6B6B6B]">
        Keep chatting — once the conversation has enough context, you'll be able
        to generate a summary here.
      </p>
    </div>
  );
}

function SummaryContent({ summary }: { summary: ApiSummary }) {
  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(summary.content);
    } catch {
      console.error("Failed to copy summary");
    }
  };

  return (
    <div className="flex flex-1 flex-col gap-5 px-5 py-5">
      <figure className="relative">
        <span
          aria-hidden="true"
          className="absolute -left-1 -top-3 select-none font-serif text-[64px] leading-none text-[#8B6F47]/25"
        >
          "
        </span>
        <blockquote
          className="rounded-xl border border-[#E8E0D0] bg-white px-5 py-4 text-[14.5px] italic leading-relaxed text-[#2C2C2C] shadow-sm"
          style={{ borderLeft: "3px solid #8B6F47", textWrap: "pretty" }}
        >
          {summary.content}
        </blockquote>
      </figure>

      <div className="rounded-xl border border-[#E8E0D0] bg-white/60 p-4 backdrop-blur">
        <div className="mb-2.5 text-[11px] font-medium uppercase tracking-wider text-[#8B6F47]">
          Details
        </div>
        <div className="flex flex-col gap-2">
          <MetaRow
            label="Messages covered"
            value={
              <span className="font-mono text-[12.5px]">
                #{summary.message_range_start}{" "}
                <span className="text-[#6B6B6B]">→</span> #
                {summary.message_range_end}
              </span>
            }
          />
          <MetaRow
            label="Generated"
            value={
              <span title={new Date(summary.created_at).toLocaleString()}>
                {relativeTime(summary.created_at)}
              </span>
            }
          />
          <MetaRow
            label="Length"
            value={
              <span className="font-mono text-[12.5px]">
                {summary.content.split(/\s+/).length} words
              </span>
            }
          />
        </div>
      </div>

      <div className="flex items-center gap-2">
        <button
          type="button"
          disabled
          title="Available in a future version"
          className="inline-flex flex-1 items-center justify-center gap-1.5 rounded-lg bg-[#8B6F47] px-3 py-2 text-[13px] font-medium text-white shadow-sm transition hover:bg-[#7a6040] disabled:cursor-not-allowed disabled:opacity-50 focus:outline-none focus-visible:ring-2 focus-visible:ring-[#8B6F47]/40"
        >
          <svg
            width="13"
            height="13"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path d="M21 12a9 9 0 1 1-3-6.7L21 8"></path>
            <path d="M21 3v5h-5"></path>
          </svg>
          Regenerate
        </button>
        <button
          type="button"
          onClick={handleCopy}
          className="inline-flex items-center justify-center gap-1.5 rounded-lg border border-[#E8E0D0] bg-white px-3 py-2 text-[13px] font-medium text-[#2C2C2C] shadow-sm transition hover:border-[#8B6F47] hover:text-[#8B6F47] focus:outline-none focus-visible:ring-2 focus-visible:ring-[#8B6F47]/40"
        >
          <svg
            width="13"
            height="13"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <rect x="9" y="9" width="13" height="13" rx="2"></rect>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
          </svg>
          Copy
        </button>
      </div>

      <p className="text-[11.5px] leading-relaxed text-[#6B6B6B]">
        Summaries are generated locally by your Ollama model. They're used to
        compact older messages so the persona keeps long conversations in mind
        without exhausting the context window.
      </p>
    </div>
  );
}

export function SummaryDrawer({
  open,
  onClose,
  conversationId,
  personaName,
}: SummaryDrawerProps) {
  const [summary, setSummary] = useState<ApiSummary | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!open) return;

    setLoading(true);
    getSummary(conversationId)
      .then(setSummary)
      .catch((err) => {
        console.error("Failed to fetch summary:", err);
        setSummary(null);
      })
      .finally(() => setLoading(false));
  }, [open, conversationId]);

  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, onClose]);

  return (
    <div
      aria-hidden={!open}
      className={`fixed inset-0 z-50 ${open ? "" : "pointer-events-none"}`}
      style={{
        fontFamily:
          'Inter, ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, sans-serif',
      }}
    >
      <div
        onClick={onClose}
        className={`absolute inset-0 bg-[#2C2C2C]/25 backdrop-blur-[1px] transition-opacity duration-300 ${open ? "opacity-100" : "opacity-0"}`}
      ></div>

      <aside
        role="dialog"
        aria-modal="true"
        aria-labelledby="summary-drawer-title"
        className={`absolute right-0 top-0 flex h-full w-full max-w-[400px] flex-col border-l border-[#E8E0D0] bg-[#F5F0E8] shadow-xl transition-transform duration-300 ease-out ${open ? "translate-x-0" : "translate-x-full"}`}
      >
        <header className="flex items-center justify-between gap-2 border-b border-[#E8E0D0] bg-[#F5F0E8]/85 px-5 py-3 backdrop-blur">
          <div className="min-w-0">
            <h2
              id="summary-drawer-title"
              className="text-[15px] font-semibold text-[#2C2C2C]"
            >
              Conversation Summary
            </h2>
            <p className="text-[11.5px] text-[#6B6B6B]">
              {summary ? `${personaName} · auto-compacted` : "Generated locally · auto-compact"}
            </p>
          </div>
          <CloseButton onClick={onClose} />
        </header>

        <div className="flex flex-1 flex-col overflow-y-auto">
          {loading ? (
            <div className="flex flex-1 items-center justify-center">
              <div className="text-[13px] text-[#6B6B6B]">Loading…</div>
            </div>
          ) : summary ? (
            <SummaryContent summary={summary} />
          ) : (
            <SummaryEmptyState />
          )}
        </div>
      </aside>
    </div>
  );
}
