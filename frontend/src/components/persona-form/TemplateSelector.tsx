import type { InstructionTemplate } from "../../types/api";
import {
  Combobox,
  ComboboxContent,
  ComboboxEmpty,
  ComboboxInput,
  ComboboxItem,
  ComboboxList,
} from "@/components/ui/combobox";

export interface TemplateSelectorProps {
  value: InstructionTemplate;
  onChange: (r: InstructionTemplate) => void;
}

export function TemplateSelector({ value, onChange }: TemplateSelectorProps) {
  const template: InstructionTemplate[] = ["default", "nsfw", "custom"];
  return (
    <Combobox
      items={template}
      value={value}
      onValueChange={(nextValue) => {
        if (nextValue) onChange(nextValue);
      }}
    >
      <ComboboxInput placeholder="Select a template" />
      <ComboboxContent>
        <ComboboxEmpty>No items found.</ComboboxEmpty>
        <ComboboxList>
          {(item) => (
            <ComboboxItem key={item} value={item}>
              {item}
            </ComboboxItem>
          )}
        </ComboboxList>
      </ComboboxContent>
    </Combobox>
  );
}
