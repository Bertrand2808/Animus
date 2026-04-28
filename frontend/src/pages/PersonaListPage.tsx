import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { ImportPersonaModal } from "../components/ImportPersonaModal";
import { OllamaStatusBadge } from "../components/OllamaStatusBadge";
import { PersonaCard } from "../components/PersonaCard";
import { useOllamaStatus } from "../hooks/useOllamaStatus";
import { usePersonas } from "../hooks/usePersonas";
import { createConversation, getLatestConversation } from "../lib/api";

export default function PersonaListPage() {
  const [query, setQuery] = useState("");
  const [importOpen, setImportOpen] = useState(false);
  const [navigating, setNavigating] = useState<string | null>(null);

  const { personas, loading, error, refetch } = usePersonas();
  const ollamaStatus = useOllamaStatus();
  const navigate = useNavigate();

  const filtered = personas.filter((p) =>
    `${p.name} ${p.description}`
      .toLowerCase()
      .includes(query.toLowerCase()),
  );

  const handlePersonaClick = async (personaId: string) => {
    setNavigating(personaId);
    try {
      const existing = await getLatestConversation(personaId);
      if (existing) {
        void navigate(`/chat/${existing.id}`);
      } else {
        const created = await createConversation(personaId);
        void navigate(`/chat/${created.id}`);
      }
    } catch {
      setNavigating(null);
    }
  };

  return (
    <div
      className="min-h-screen w-full"
      style={{
        backgroundColor: "#F5F0E8",
        fontFamily:
          'Inter, ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, sans-serif',
        color: "#2C2C2C",
      }}
    >
      {/* Top bar */}
      <header className="sticky top-0 z-20 border-b border-[#E8E0D0] bg-[#F5F0E8]/85 backdrop-blur">
        <div className="mx-auto flex max-w-6xl items-center justify-between px-4 py-3 sm:px-6 lg:px-8">
          <div className="flex items-center gap-2.5">
            <div className="grid h-7 w-7 place-items-center rounded-md bg-[#8B6F47] text-white">
              <span className="font-serif text-[15px] leading-none">A</span>
            </div>
            <span className="text-[18px] font-semibold tracking-tight text-[#2C2C2C]">
              Animus
            </span>
          </div>

          <div className="flex items-center gap-2 sm:gap-3">
            <div className="hidden sm:block">
              <OllamaStatusBadge
                online={ollamaStatus.online}
                model={ollamaStatus.model}
              />
            </div>
            <button
              aria-label="Settings"
              className="grid h-9 w-9 place-items-center rounded-lg text-[#6B6B6B] transition hover:bg-white hover:text-[#2C2C2C] focus:outline-none focus-visible:ring-2 focus-visible:ring-[#8B6F47]/40"
            >
              <svg
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.8"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
                <circle cx="12" cy="12" r="3" />
              </svg>
            </button>
          </div>
        </div>
      </header>

      {/* Main */}
      <main className="mx-auto max-w-6xl px-4 pb-32 pt-8 sm:px-6 lg:px-8">
        <div className="mb-6 flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <h1 className="text-[28px] font-semibold tracking-tight text-[#2C2C2C]">
              Your personas
            </h1>
            <p className="mt-1 text-[14px] text-[#6B6B6B]">
              {loading
                ? "Loading…"
                : `${personas.length} character${personas.length !== 1 ? "s" : ""} · Running locally`}
            </p>
          </div>

          <div className="relative w-full max-w-xs">
            <svg
              className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-[#6B6B6B]"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <circle cx="11" cy="11" r="8" />
              <path d="m21 21-4.3-4.3" />
            </svg>
            <input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search personas"
              className="w-full rounded-lg border border-[#E8E0D0] bg-white py-2 pl-9 pr-3 text-[14px] text-[#2C2C2C] placeholder:text-[#6B6B6B] focus:border-[#8B6F47] focus:outline-none focus:ring-2 focus:ring-[#8B6F47]/20"
            />
          </div>
        </div>

        <div className="mb-4 sm:hidden">
          <OllamaStatusBadge
            online={ollamaStatus.online}
            model={ollamaStatus.model}
          />
        </div>

        {error && (
          <div className="mb-4 rounded-lg border border-rose-200 bg-rose-50 px-4 py-3 text-[14px] text-rose-700">
            {error}
          </div>
        )}

        {loading ? (
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
            {[1, 2, 3].map((i) => (
              <div
                key={i}
                className="h-36 animate-pulse rounded-xl border border-[#E8E0D0] bg-white"
              />
            ))}
          </div>
        ) : filtered.length > 0 ? (
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
            {filtered.map((p) => (
              <div key={p.id} className={navigating === p.id ? "opacity-60" : ""}>
                <PersonaCard
                  persona={p}
                  onClick={() => void handlePersonaClick(p.id)}
                />
              </div>
            ))}
          </div>
        ) : (
          <div className="rounded-xl border border-dashed border-[#E8E0D0] bg-white/40 p-12 text-center">
            <p className="text-[14px] text-[#6B6B6B]">
              {query
                ? `No personas match "${query}".`
                : "No personas yet. Import one to get started."}
            </p>
          </div>
        )}
      </main>

      {/* Floating import button */}
      <button
        aria-label="Import new persona"
        onClick={() => setImportOpen(true)}
        className="group fixed bottom-6 right-6 inline-flex items-center gap-2 rounded-full bg-[#8B6F47] py-3 pl-4 pr-5 text-white shadow-lg shadow-[#8B6F47]/25 transition hover:bg-[#7a6040] focus:outline-none focus-visible:ring-2 focus-visible:ring-[#8B6F47]/40 sm:bottom-8 sm:right-8"
      >
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2.2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <path d="M12 5v14" />
          <path d="M5 12h14" />
        </svg>
        <span className="text-[14px] font-medium">Import persona</span>
      </button>

      {importOpen && (
        <ImportPersonaModal
          onClose={() => setImportOpen(false)}
          onImported={refetch}
        />
      )}
    </div>
  );
}
