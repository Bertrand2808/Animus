import { useEffect, useState } from "react";
import { getOllamaStatus } from "../lib/api";
import type { OllamaStatus } from "../types/api";

const POLL_INTERVAL_MS = 30_000;

export function useOllamaStatus(): OllamaStatus {
  const [status, setStatus] = useState<OllamaStatus>({
    online: false,
    model: "",
  });

  useEffect(() => {
    const poll = () => {
      getOllamaStatus()
        .then(setStatus)
        .catch(() => setStatus((prev) => ({ ...prev, online: false })));
    };

    poll();
    const id = setInterval(poll, POLL_INTERVAL_MS);
    return () => clearInterval(id);
  }, []);

  return status;
}
