import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { useSettings } from "@/hooks/useSettings";
import { AlertCircle, Check, Loader2 } from "lucide-react";

export default function SettingsPage() {
  const {
    formData,
    setFormData,
    save,
    isSaving,
    isSuccess,
    isDirty,
    loading,
    error,
    saveError,
  } = useSettings();

  if (loading) {
    return (
      <div className="min-h-screen bg-[#F8F8F6] px-6 py-10 text-[#2B2721] md:px-10">
        <div className="mx-auto flex max-w-3xl items-center gap-3 text-sm text-[#6F685F]">
          <Loader2 className="h-4 w-4 animate-spin" />
          Loading settings...
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen bg-[#F8F8F6] px-6 py-10 text-[#2B2721] md:px-10">
        <div className="mx-auto max-w-3xl rounded-lg border border-[#E3B7AA] bg-[#FFF8F5] p-4 text-sm text-[#9A3412]">
          <div className="flex items-start gap-3">
            <AlertCircle className="mt-0.5 h-4 w-4 shrink-0" />
            <div>
              <p className="font-medium">Settings unavailable</p>
              <p className="mt-1 text-[#9A3412]/80">{error}</p>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-[#F8F8F6] px-6 py-10 text-[#2B2721] md:px-10">
      <div className="mx-auto max-w-3xl">
        <div className="mb-8">
          <h1 className="font-heading text-3xl font-medium tracking-normal">
            Settings
          </h1>
          <p className="mt-2 max-w-xl text-sm leading-6 text-[#6F685F]">
            Adjust how Animus identifies you and which model it uses by default.
          </p>
        </div>

        <Card className="rounded-lg border-[#D8D3C8] bg-[#FFFEFB] shadow-none">
          <CardHeader className="px-6 pt-6">
            <CardTitle className="text-lg text-[#2B2721]">
              Account information
            </CardTitle>
            <CardDescription className="text-[#6F685F]">
              Changes are applied after saving.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-6 px-6 pb-6 pt-2">
            <Separator className="bg-[#E6E0D8]" />

            <div className="grid gap-3 md:grid-cols-[180px_1fr] md:gap-6">
              <div>
                <Label htmlFor="user_name" className="text-[#2B2721]">
                  User name
                </Label>
                <p className="mt-1 text-sm leading-5 text-[#6F685F]">
                  How characters address you.
                </p>
              </div>
              <Input
                id="user_name"
                value={formData.user_name}
                onChange={(e) => setFormData({ user_name: e.target.value })}
                placeholder="John Doe"
                className="h-10 max-w-sm border-[#D8D3C8] bg-[#FDFCF9] text-[#2B2721] placeholder:text-[#9C9489] focus-visible:border-[#C15F3C] focus-visible:ring-[#C15F3C]/20"
              />
            </div>

            <Separator className="bg-[#E6E0D8]" />

            <div className="grid gap-3 md:grid-cols-[180px_1fr] md:gap-6">
              <div>
                <Label htmlFor="default_model" className="text-[#2B2721]">
                  Default model
                </Label>
                <p className="mt-1 text-sm leading-5 text-[#6F685F]">
                  The model Animus starts with.
                </p>
              </div>
              <Input
                id="default_model"
                value={formData.default_model}
                onChange={(e) => setFormData({ default_model: e.target.value })}
                placeholder="gemma4"
                className="h-10 max-w-sm border-[#D8D3C8] bg-[#FDFCF9] text-[#2B2721] placeholder:text-[#9C9489] focus-visible:border-[#C15F3C] focus-visible:ring-[#C15F3C]/20"
              />
            </div>

            <Separator className="bg-[#E6E0D8]" />

            <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
              <div className="min-h-5 text-sm">
                {saveError && (
                  <p className="flex items-center gap-2 text-[#B42318]">
                    <AlertCircle className="h-4 w-4" />
                    {saveError}
                  </p>
                )}
                {isSuccess && !saveError && (
                  <p className="flex items-center gap-2 text-[#287A52]">
                    <Check className="h-4 w-4" />
                    Settings updated
                  </p>
                )}
                {!isDirty && !isSaving && !isSuccess && !saveError && (
                  <p className="text-[#8A8176]">No pending changes</p>
                )}
              </div>

              <Button
                onClick={save}
                disabled={!isDirty || isSaving}
                size="lg"
                className="h-10 rounded-lg bg-[#C15F3C] px-4 text-sm font-medium text-white shadow-none transition-colors hover:bg-[#AA5234] active:bg-[#91462D] disabled:cursor-not-allowed disabled:bg-[#D8D3C8] disabled:text-[#7B746A] focus-visible:ring-[#C15F3C]/30"
              >
                {isSaving && (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Saving...
                  </>
                )}

                {isSuccess && !isSaving && (
                  <>
                    <Check className="mr-2 h-4 w-4" />
                    Saved
                  </>
                )}

                {!isSaving && !isSuccess && "Save changes"}
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
