interface Props {
  online: boolean;
  model: string;
}

export function OllamaStatusBadge({ online, model }: Props) {
  return (
    <div className="inline-flex items-center gap-2 rounded-full border border-[#E8E0D0] bg-white px-3 py-1.5 text-[12px] text-[#2C2C2C] shadow-sm">
      <span className="relative flex h-2 w-2">
        <span
          className={`absolute inline-flex h-full w-full rounded-full opacity-60 ${online ? "animate-ping bg-emerald-400" : "bg-zinc-300"}`}
        />
        <span
          className={`relative inline-flex h-2 w-2 rounded-full ${online ? "bg-emerald-500" : "bg-zinc-400"}`}
        />
      </span>
      <span className="text-[#6B6B6B]">Ollama</span>
      {model && (
        <span className="font-mono text-[11px] text-[#2C2C2C]">{model}</span>
      )}
    </div>
  );
}
