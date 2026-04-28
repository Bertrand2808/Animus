export interface SectionCardProps {
  step: number;
  title: string;
  hint?: string;
  children: React.ReactNode;
}

export function SectionCard({ step, title, hint, children }: SectionCardProps) {
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
