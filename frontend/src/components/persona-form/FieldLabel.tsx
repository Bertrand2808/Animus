export const inputClass =
  "w-full rounded-lg border border-[#E8E0D0] bg-[#FAFAF6] px-3 py-2 text-[14px] text-[#2C2C2C] placeholder:text-[#6B6B6B]/70 transition focus:border-[#8B6F47] focus:bg-white focus:outline-none focus:ring-2 focus:ring-[#8B6F47]/15";

export interface FieldLabelProps {
  label: string;
  required?: boolean;
  htmlFor?: string;
  hint?: React.ReactNode;
}

export function FieldLabel({ label, required, htmlFor, hint }: FieldLabelProps) {
  return (
    <div className="mb-1.5 flex items-baseline justify-between gap-3">
      <label
        htmlFor={htmlFor}
        className="text-[13px] font-medium text-[#2C2C2C]"
      >
        {label}
        {required && <span className="ml-1 text-[#B05546]">*</span>}
      </label>
      {hint && <span className="text-[11.5px] text-[#6B6B6B]">{hint}</span>}
    </div>
  );
}
