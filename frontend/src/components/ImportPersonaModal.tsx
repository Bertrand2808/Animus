import { useRef, useState } from "react";
import { importPersona } from "../lib/api";

interface Props {
  onClose: () => void;
  onImported: () => void;
}

type Status = "idle" | "loading" | "error";

export function ImportPersonaModal({ onClose, onImported }: Props) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [status, setStatus] = useState<Status>("idle");
  const [errorMsg, setErrorMsg] = useState("");

  const handleFile = async (file: File) => {
    setStatus("loading");
    setErrorMsg("");
    try {
      const text = await file.text();
      await importPersona(text);
      onImported();
      onClose();
    } catch (err) {
      setStatus("error");
      setErrorMsg(err instanceof Error ? err.message : "Import failed");
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) void handleFile(file);
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/30 p-4"
      onClick={(e) => e.target === e.currentTarget && onClose()}
    >
      <div className="w-full max-w-md rounded-xl border border-[#E8E0D0] bg-[#F5F0E8] p-6 shadow-lg">
        <h2 className="text-[18px] font-semibold text-[#2C2C2C]">
          Import persona
        </h2>
        <p className="mt-1 text-[13px] text-[#6B6B6B]">
          Select a Character Card V2 JSON file.
        </p>

        <input
          ref={inputRef}
          type="file"
          accept=".json,application/json"
          className="hidden"
          onChange={handleChange}
        />

        <button
          type="button"
          disabled={status === "loading"}
          onClick={() => inputRef.current?.click()}
          className="mt-5 w-full rounded-lg border-2 border-dashed border-[#E8E0D0] bg-white py-8 text-[14px] text-[#6B6B6B] transition hover:border-[#8B6F47] hover:text-[#8B6F47] disabled:opacity-50"
        >
          {status === "loading" ? "Importing…" : "Click to choose a file"}
        </button>

        {status === "error" && (
          <p className="mt-3 text-[13px] text-rose-600">{errorMsg}</p>
        )}

        <div className="mt-5 flex justify-end">
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg px-4 py-2 text-[14px] text-[#6B6B6B] hover:bg-white"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}
