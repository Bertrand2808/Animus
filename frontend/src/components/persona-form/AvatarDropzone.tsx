import { useRef, useState } from "react";
import { readAsDataUrl } from "../../lib/readAsDataUrl";
import { FieldLabel } from "./FieldLabel";

export interface AvatarDropzoneProps {
  value: string | undefined;
  onChange: (v: string | undefined) => void;
}

export function AvatarDropzone({ value, onChange }: AvatarDropzoneProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [over, setOver] = useState(false);

  const handleFiles = async (files: FileList | null) => {
    if (!files?.[0]) return;
    const url = await readAsDataUrl(files[0]);
    onChange(url);
  };

  return (
    <div className="flex items-center gap-5">
      <button
        type="button"
        onClick={() => inputRef.current?.click()}
        onDragOver={(e) => { e.preventDefault(); setOver(true); }}
        onDragLeave={() => setOver(false)}
        onDrop={(e) => { e.preventDefault(); setOver(false); void handleFiles(e.dataTransfer.files); }}
        className={[
          "group relative grid h-[120px] w-[120px] shrink-0 place-items-center overflow-hidden rounded-full border-2 border-dashed transition",
          over
            ? "border-[#8B6F47] bg-[#F5F0E8]"
            : "border-[#C9BCA6] bg-[#FAFAF6] hover:border-[#8B6F47] hover:bg-[#F5F0E8]",
        ].join(" ")}
        aria-label="Upload avatar image"
      >
        {value ? (
          <>
            <img src={value} alt="Avatar preview" className="h-full w-full object-cover" />
            <span className="absolute inset-0 grid place-items-center bg-black/40 text-[12px] font-medium text-white opacity-0 transition group-hover:opacity-100">
              Replace
            </span>
          </>
        ) : (
          <div className="flex flex-col items-center gap-1.5 text-center text-[#6B6B6B]">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
              <polyline points="17 8 12 3 7 8" />
              <line x1="12" y1="3" x2="12" y2="15" />
            </svg>
            <span className="text-[11px] font-medium">Click or drag</span>
          </div>
        )}
      </button>

      <div className="min-w-0">
        <FieldLabel label="Avatar" hint="Square image, 80–512px" />
        <p className="text-[12.5px] leading-relaxed text-[#6B6B6B]">
          Shown on the persona card and in chat. PNG or JPG.
        </p>
        {value && (
          <button
            type="button"
            onClick={() => onChange(undefined)}
            className="mt-2 text-[12px] font-medium text-[#8B6F47] underline-offset-2 hover:underline"
          >
            Remove
          </button>
        )}
      </div>

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
