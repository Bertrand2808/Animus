import { useCallback, useEffect, useState } from "react";
import { listPersonas } from "../lib/api";
import type { Persona } from "../types/api";

interface UsePersonasResult {
  personas: Persona[];
  loading: boolean;
  error: string | null;
  refetch: () => void;
}

export function usePersonas(): UsePersonasResult {
  const [personas, setPersonas] = useState<Persona[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetch = useCallback(() => {
    setLoading(true);
    setError(null);
    listPersonas()
      .then(setPersonas)
      .catch((err: unknown) =>
        setError(err instanceof Error ? err.message : "Unknown error"),
      )
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    fetch();
  }, [fetch]);

  return { personas, loading, error, refetch: fetch };
}
