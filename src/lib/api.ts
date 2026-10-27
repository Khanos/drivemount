import { invoke, isTauri } from "@tauri-apps/api/core";
import type { AppError, Commands } from "./types";
export const isMock = import.meta.env.MODE === "mock";
export async function api<K extends keyof Commands>(
  command: K,
  args?: Commands[K]["args"],
): Promise<Commands[K]["result"]> {
  if (isMock) {
    const { mockInvoke } = await import("./mock");
    return mockInvoke(command, args);
  }
  if (!isTauri())
    throw {
      code: "desktop_required",
      message:
        "Open the installed DriveMount app to connect your computer. Browser previews require the explicit mock development mode.",
      recoverable: false,
    } satisfies AppError;
  try {
    return await invoke<Commands[K]["result"]>(
      command,
      args as Record<string, unknown> | undefined,
    );
  } catch (error) {
    throw asError(error);
  }
}
export function asError(error: unknown): AppError {
  if (typeof error === "object" && error !== null && "message" in error)
    return {
      code: "code" in error ? String(error.code) : "operation",
      message: String(error.message),
      recoverable: true,
    };
  return {
    code: "operation",
    message:
      "This operation could not finish. Refresh your connection details and try again.",
    recoverable: true,
  };
}
