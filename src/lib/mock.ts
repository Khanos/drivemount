import type {
  AuthStatus,
  Commands,
  Connection,
  ConnectionStatus,
  Inspection,
  Preferences,
} from "./types";
import { defaults } from "./store";
// Explicit UI development only. No host commands, OAuth requests, or filesystem changes.
const scenario =
  new URLSearchParams(window.location.search).get("scenario") ?? "fresh";
let connections: Connection[] = JSON.parse(
  localStorage.getItem("drivemount-mock-connections") ?? "[]",
);
const connected = new Set(
  connections.filter((c) => c.configured).map((c) => c.id),
);
let auth: AuthStatus | null = null;
let authAt = 0;
let installed = false;
const save = () =>
  localStorage.setItem(
    "drivemount-mock-connections",
    JSON.stringify(connections),
  );
const fail = (code: string, message: string): never => {
  throw { code, message, recoverable: true };
};
const connection = (id: string) =>
  connections.find((c) => c.id === id) ??
  fail("missing", "Preview connection not found.");
const draft = (remote = "drivemount_preview"): Connection => ({
  id: crypto.randomUUID(),
  remote,
  configPath: "/home/demo/.config/drivemount/preview/rclone.conf",
  mountPath: "/home/demo/GoogleDrive",
  service: "drivemount-preview.service",
  cachePath: "/home/demo/.cache/drivemount/preview",
  socketPath: "/run/user/1000/drivemount/preview.sock",
  managed: true,
  ownedRemote: true,
  authorized: false,
  configured: false,
  preferences: structuredClone(defaults),
});
const status = (id: string): ConnectionStatus => ({
  id,
  state:
    scenario === "offline"
      ? "offline"
      : connected.has(id)
        ? "connected"
        : "disconnected",
  mounted: connected.has(id),
  activeState: connected.has(id) ? "active" : "inactive",
  autostart: connection(id).preferences.autostart,
  pendingUploads: scenario === "pending" ? 2 : 0,
  openFiles: 0,
  cacheBytes: 124 * 1024 * 1024,
  detail:
    scenario === "offline"
      ? "You're offline. Drive will reconnect when a connection is available."
      : connected.has(id)
        ? "Your Drive folder is mounted."
        : "Your Drive is disconnected.",
});
export async function mockInvoke<K extends keyof Commands>(
  command: K,
  args: Commands[K]["args"],
): Promise<Commands[K]["result"]> {
  await new Promise((resolve) => setTimeout(resolve, 100));
  let result: unknown;
  const a = args as Record<string, unknown> | undefined;
  switch (command) {
    case "inspect_system": {
      const missing = scenario === "missing" && !installed;
      result = {
        distro: "Ubuntu 24.04 LTS",
        packageManager: "apt",
        packages: missing ? ["rclone", "fuse3"] : [],
        checks: [
          {
            id: "platform",
            label: "Linux distribution",
            state: "ready",
            detail: "Ubuntu 24.04 LTS",
          },
          {
            id: "rclone",
            label: "Google Drive connector",
            state: missing ? "install" : "ready",
            detail: missing
              ? "Rclone needs to be installed."
              : "rclone v1.75.2",
          },
          {
            id: "fuse",
            label: "Filesystem support",
            state: missing ? "install" : "ready",
            detail: "FUSE3 mounting support",
          },
          {
            id: "fuse_device",
            label: "Filesystem device access",
            state: "ready",
            detail: "Your account has permission.",
          },
          {
            id: "systemd",
            label: "Automatic connection support",
            state: "ready",
            detail: "Your desktop service manager is ready.",
          },
          {
            id: "internet",
            label: "Internet connection",
            state: scenario === "offline" ? "attention" : "ready",
            detail:
              scenario === "offline"
                ? "Google cannot be reached."
                : "Google's servers can be reached.",
          },
        ],
        remotes:
          scenario === "existing"
            ? [
                {
                  name: "gdrive",
                  backend: "drive",
                  configPath: "/home/demo/.config/rclone/rclone.conf",
                },
              ]
            : [],
        mounts:
          scenario === "existing"
            ? [
                {
                  target: "/home/demo/GoogleDrive",
                  source: "gdrive:",
                  service: "rclone-gdrive.service",
                },
              ]
            : [],
        services: [],
        connections,
        defaultMount: "/home/demo/GoogleDrive",
        appOauthAvailable: false,
        freeCacheBytes: 80 * 1024 ** 3,
      } satisfies Inspection;
      break;
    }
    case "install_dependencies":
      if (!a?.consent) fail("consent", "Approve installation first.");
      installed = true;
      break;
    case "begin_auth": {
      const req = a?.request as { connectionId: string | null };
      const c = req.connectionId ? connection(req.connectionId) : draft();
      if (!req.connectionId) {
        connections = [...connections, c];
        save();
      }
      auth = {
        id: c.id,
        state: "waiting",
        message: "Waiting for Google authorization in your browser…",
      };
      authAt = Date.now();
      result = auth;
      break;
    }
    case "auth_status":
      if (auth?.state === "waiting" && Date.now() - authAt > 1400) {
        auth = {
          ...auth,
          state: scenario === "oauth-failure" ? "failed" : "connected",
          message:
            scenario === "oauth-failure"
              ? "Google authorization failed. Check your credentials and try again."
              : "Your Google account is connected.",
        };
        if (auth.state === "connected") {
          connection(auth.id).authorized = true;
          save();
        }
      }
      result = auth;
      break;
    case "cancel_auth":
      if (auth)
        auth = {
          ...auth,
          state: "cancelled",
          message:
            "Google account connection was cancelled. You can try again.",
        };
      break;
    case "validate_mount_path": {
      const path = String(a?.path);
      const valid =
        path.startsWith("/") &&
        !path.includes("..") &&
        !path.includes("populated");
      result = {
        path,
        valid,
        exists: false,
        reason: valid
          ? "An empty, accessible folder is ready for your Drive."
          : "Choose an empty folder using an absolute path.",
      };
      break;
    }
    case "import_remote": {
      if (!a?.consent) fail("consent", "Approve management first.");
      let c = draft(String(a?.name));
      c = {
        ...c,
        ownedRemote: false,
        authorized: true,
        managed: scenario !== "existing",
        configured: scenario === "existing",
        service: "rclone-gdrive.service",
      };
      connections = [...connections, c];
      if (c.configured) connected.add(c.id);
      save();
      result = c;
      break;
    }
    case "connect_drive": {
      const req = a?.request as {
        id: string;
        mountPath: string;
        preferences: Preferences;
      };
      if (scenario === "mount-failure")
        fail(
          "mount_failed",
          "Google Drive did not appear at the chosen folder. Refresh diagnostics and try again.",
        );
      let c = connection(req.id);
      c = {
        ...c,
        mountPath: req.mountPath,
        preferences: req.preferences,
        configured: true,
      };
      connections = connections.map((old) => (old.id === c.id ? c : old));
      connected.add(c.id);
      save();
      result = status(c.id);
      break;
    }
    case "setup_progress":
      result = { stage: "mount", message: "Connecting your Google Drive…" };
      break;
    case "connection_status":
      result = status(String(a?.id));
      break;
    case "connection_action": {
      const id = String(a?.id);
      if (a?.action === "stop" && scenario === "pending")
        fail(
          "pending_writes",
          "Drive has unfinished uploads. Wait for uploads before disconnecting.",
        );
      if (a?.action === "stop") connected.delete(id);
      else connected.add(id);
      result = status(id);
      break;
    }
    case "set_autostart":
      connection(String(a?.id)).preferences.autostart = Boolean(a?.enabled);
      save();
      break;
    case "read_diagnostics":
      result = {
        report:
          "DriveMount UI preview\nService: active\nMount detected: true\nNo real system actions were performed.",
        details: "Local troubleshooting preview. Review before sharing.",
      };
      break;
    case "choose_folder":
      result = "/home/demo/My Drive";
      break;
    case "open_drive":
    case "open_help":
      break;
  }
  return result as Commands[K]["result"];
}
