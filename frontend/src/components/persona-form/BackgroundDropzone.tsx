import { useRef, useState } from "react";
import { readAsDataUrl } from "../../lib/readAsDataUrl";
import { FieldLabel } from "./FieldLabel";

export interface BackgroundDropzoneProps {
  value: string | undefined;
  onChange: (v: string | undefined) => void;
}

export function BackgroundDropzone({ value, onChange }: BackgroundDropzoneProps) {
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
