import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { Preferences } from "./types";
export const defaults: Preferences = {
  autostart: true,
  readOnly: false,
  cache: { mode: "full", sizeGib: 10, ageHours: 24 },
};
type Store = {
  theme: "system" | "light" | "dark";
  mountPath: string;
  preferences: Preferences;
  selectedId: string | null;
  setTheme: (t: Store["theme"]) => void;
  setMountPath: (p: string) => void;
  setPreferences: (p: Preferences) => void;
  select: (id: string | null) => void;
};
export const useSettings = create<Store>()(
  persist(
    (set) => ({
      theme: "system",
      mountPath: "",
      preferences: structuredClone(defaults),
      selectedId: null,
      setTheme: (theme) => set({ theme }),
      setMountPath: (mountPath) => set({ mountPath }),
      setPreferences: (preferences) => set({ preferences }),
      select: (selectedId) => set({ selectedId }),
    }),
    { name: "drivemount-preferences-v1", version: 1 },
  ),
);
export const presets = {
  "Save disk space": { mode: "writes", sizeGib: 2, ageHours: 6 },
  Balanced: { mode: "full", sizeGib: 10, ageHours: 24 },
  "High performance": { mode: "full", sizeGib: 30, ageHours: 72 },
} as const;
export function credentialError(
  clientId: string,
  secret: string,
): string | null {
  if (
    !clientId.trim().endsWith(".apps.googleusercontent.com") ||
    !/^[a-zA-Z0-9._-]+$/.test(clientId) ||
    secret.trim().length < 8
  )
    return "Enter your Desktop app client ID and client secret from Google Cloud.";
  return null;
}
