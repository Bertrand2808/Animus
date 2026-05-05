import { getSettings, updateSettings } from "@/lib/api";
import type { PatchSettingsRequest, SettingsResponse } from "@/types/api";
import { useCallback, useEffect, useMemo, useState } from "react";

type SettingsFormData = Pick<SettingsResponse, "user_name" | "default_model">;

interface UseSettingsResult {
  settings: SettingsResponse | null;
  loading: boolean;
  error: string | null;
  isDirty: boolean;
  formData: SettingsFormData;
  setFormData: (partial: Partial<SettingsFormData>) => void;
  isSaving: boolean;
  isSuccess: boolean;
  saveError: string | null;
  refetch: () => void;
  save: () => Promise<void>;
}

const EMPTY_SETTINGS_FORM: SettingsFormData = {
  user_name: "",
  default_model: "",
};

export function useSettings(): UseSettingsResult {
  const [settings, setSettings] = useState<SettingsResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [formData, setFormDataState] =
    useState<SettingsFormData>(EMPTY_SETTINGS_FORM);
  const [isSaving, setIsSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [isSuccess, setIsSuccess] = useState(false);

  const fetchSettings = useCallback(() => {
    setLoading(true);
    setError(null);

    getSettings()
      .then((data) => {
        setSettings(data);
        setFormDataState({
          user_name: data.user_name,
          default_model: data.default_model,
        });
      })
      .catch((err: unknown) =>
        setError(err instanceof Error ? err.message : "Unknown error"),
      )
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    let ignore = false;

    getSettings()
      .then((data) => {
        if (ignore) {
          return;
        }

        setSettings(data);
        setFormDataState({
          user_name: data.user_name,
          default_model: data.default_model,
        });
      })
      .catch((err: unknown) => {
        if (ignore) {
          return;
        }

        setError(err instanceof Error ? err.message : "Unknown error");
      })
      .finally(() => {
        if (!ignore) {
          setLoading(false);
        }
      });

    return () => {
      ignore = true;
    };
  }, []);

  const setFormData = useCallback((partial: Partial<SettingsFormData>) => {
    setFormDataState((current) => ({ ...current, ...partial }));
    setSaveError(null);
  }, []);

  const isDirty = useMemo(() => {
    if (!settings) {
      return false;
    }
    return (
      formData.user_name !== settings.user_name ||
      formData.default_model !== settings.default_model
    );
  }, [formData, settings]);

  const save = useCallback(async () => {
    const userName = formData.user_name.trim();
    if (!userName) {
      setSaveError("User name can not be empty.");
      return;
    }
    setIsSaving(true);
    setSaveError(null);
    setIsSuccess(false);
    try {
      const payload: PatchSettingsRequest = {
        user_name: userName,
        default_model: formData.default_model,
      };
      const updated = await updateSettings(payload);
      setSettings(updated);
      setFormDataState({
        user_name: updated.user_name,
        default_model: updated.default_model,
      });
      setIsSuccess(true);
      setTimeout(() => setIsSuccess(false), 1800);
    } catch (err: unknown) {
      setSaveError(err instanceof Error ? err.message : "Unknown Error");
    } finally {
      setIsSaving(false);
    }
  }, [formData]);

  return {
    settings,
    loading,
    error,
    isDirty,
    formData,
    setFormData,
    isSaving,
    isSuccess,
    saveError,
    save,
    refetch: fetchSettings,
  };
}
