import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const source = readFileSync(join(__dirname, "../src/pages/SettingsPage.tsx"), "utf8");

const expectations = [
  ["warm app background", "bg-[#F8F8F6]"],
  ["restrained content width", "max-w-3xl"],
  ["warm neutral text", "text-[#2B2721]"],
  ["warm card border", "border-[#D8D3C8]"],
  ["section separators", "<Separator"],
  ["clear saved state copy", "Settings updated"],
];

const missing = expectations
  .filter(([, token]) => !source.includes(token))
  .map(([label, token]) => `- ${label}: ${token}`);

if (missing.length > 0) {
  throw new Error(`Settings page design expectations missing:\n${missing.join("\n")}`);
}
