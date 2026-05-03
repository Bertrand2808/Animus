import type { ResponseStylePreset } from "@/types/api";

export const RESPONSE_STYLE_PRESETS = {
  balanced: { temperature: 0.8, repeatPenalty: 1.08 },
  creative: { temperature: 1.0, repeatPenalty: 1.05 },
  stable: { temperature: 0.65, repeatPenalty: 1.12 },
} as const satisfies Record<
  ResponseStylePreset,
  { temperature: number; repeatPenalty: number }
>;

export interface ResponseLengthExample {
  label: string;
  description: string;
  sample: string;
}

export function inferResponseStylePreset(
  temperature: number,
  repeatPenalty: number,
): ResponseStylePreset | undefined {
  return (
    Object.entries(RESPONSE_STYLE_PRESETS).find(
      ([, config]) =>
        config.temperature === temperature &&
        config.repeatPenalty === repeatPenalty,
    )?.[0] as ResponseStylePreset | undefined
  );
}

export function getResponseLengthExample(length: number): ResponseLengthExample {
  if (length <= 600) {
    return {
      label: "Court",
      description: "Réponse concise, utile pour un échange rapide.",
      sample:
        "Bien sûr ! Je peux t'aider à organiser ton voyage à Marseille. Quelles sont tes dates ?",
    };
  }
  if (length <= 1200) {
    return {
      label: "Equilibre",
      description: "Assez de contexte sans ralentir la conversation.",
      sample:
        "Bien sûr ! Marseille mêle histoire, mer et gastronomie. Commence par le Vieux-Port, puis Notre-Dame de la Garde pour la vue. Tu as déjà un budget ou des dates ?",
    };
  }
  if (length <= 2000) {
    return {
      label: "Detaille",
      description: "Réponse riche, avec recommandations et nuances.",
      sample:
        "Absolument. Marseille combine culture provençale, histoire et bord de mer. Prévois le Vieux-Port, le Mucem, Notre-Dame de la Garde, puis les Calanques si tu veux une journée nature.",
    };
  }
  return {
    label: "Etendu",
    description: "Format long pour plans complets et scènes développées.",
    sample:
      "Je vais te préparer un plan complet pour ton séjour à Marseille : arrivée, quartiers à explorer, pauses repas, transports, options selon météo, puis variantes si tu veux plus de mer ou de musées.",
  };
}
