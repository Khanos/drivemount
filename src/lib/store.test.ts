import { afterEach, describe, expect, it } from "vitest";
import { credentialError, defaults, presets, useSettings } from "./store";
describe("preferences and credential validation", () => {
  afterEach(() => {
    localStorage.clear();
    useSettings.setState({
      theme: "system",
      mountPath: "",
      preferences: structuredClone(defaults),
      selectedId: null,
    });
  });
  it("keeps credentials out of persisted preferences", () => {
    useSettings.getState().setMountPath("/home/demo/My Drive 文");
    useSettings.getState().setPreferences({ ...defaults, autostart: false });
    const saved = localStorage.getItem("drivemount-preferences-v1")!;
    expect(saved).toContain("My Drive 文");
    expect(saved).not.toMatch(/clientSecret|access_token|refresh_token/);
    expect(JSON.parse(saved).state.preferences.autostart).toBe(false);
  });
  it("rejects non-Google IDs and missing secrets", () => {
    expect(credentialError("bad", "secret")).toBeTruthy();
    expect(
      credentialError("test.apps.googleusercontent.com", "long-enough-secret"),
    ).toBeNull();
  });
  it("maps balanced to the proven rclone settings", () => {
    expect(presets.Balanced).toEqual({
      mode: "full",
      sizeGib: 10,
      ageHours: 24,
    });
    expect(presets["Save disk space"].mode).toBe("writes");
  });
});
