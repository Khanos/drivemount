import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { Wizard } from "./Wizard";
import { defaults, useSettings } from "../lib/store";
import type { Inspection } from "../lib/types";
const mocked = vi.hoisted(() => ({ api: vi.fn() }));
vi.mock("../lib/api", () => ({
  api: mocked.api,
  asError: (e: unknown) => e,
  isMock: true,
}));
const ready: Inspection = {
  distro: "Test Linux",
  packageManager: "apt",
  packages: [],
  checks: [
    {
      id: "rclone",
      label: "Google Drive connector",
      state: "ready",
      detail: "Ready",
    },
  ],
  remotes: [],
  mounts: [],
  services: [],
  connections: [],
  defaultMount: "/home/test/GoogleDrive",
  appOauthAvailable: false,
  freeCacheBytes: 50 * 1024 ** 3,
};
const props = () => ({
  inspection: ready,
  onRefresh: vi.fn().mockResolvedValue(ready),
  onDashboard: vi.fn(),
  onError: vi.fn(),
  initialId: null,
});
beforeEach(() => {
  useSettings.setState({
    mountPath: "/home/test/GoogleDrive",
    preferences: structuredClone(defaults),
    selectedId: null,
  });
  mocked.api.mockReset();
  mocked.api.mockImplementation((cmd: string) =>
    cmd === "validate_mount_path"
      ? Promise.resolve({
          path: "/home/test/GoogleDrive",
          valid: true,
          exists: false,
          reason: "Ready",
        })
      : Promise.resolve(null),
  );
});
afterEach(cleanup);
describe("setup wizard", () => {
  it("navigates back without resetting preferences", async () => {
    const user = userEvent.setup();
    render(<Wizard {...props()} />);
    await user.click(screen.getByRole("button", { name: /get started/i }));
    expect(
      screen.getByRole("heading", { name: /good home/i }),
    ).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Continue" }));
    expect(
      screen.getByRole("heading", { name: /connect your google account/i }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Continue" })).toBeDisabled();
    await user.click(screen.getByRole("button", { name: "Back" }));
    expect(useSettings.getState().preferences).toEqual(defaults);
  });
  it("blocks system progression when dependencies are missing", async () => {
    const user = userEvent.setup();
    render(
      <Wizard
        {...props()}
        inspection={{
          ...ready,
          packages: ["rclone"],
          checks: [{ ...ready.checks[0], state: "install" }],
        }}
      />,
    );
    await user.click(screen.getByRole("button", { name: /get started/i }));
    expect(screen.getByRole("button", { name: "Continue" })).toBeDisabled();
    await user.click(
      screen.getByRole("button", { name: "Install Required Components" }),
    );
    expect(screen.getByRole("dialog")).toBeInTheDocument();
    expect(mocked.api).not.toHaveBeenCalledWith(
      "install_dependencies",
      expect.anything(),
    );
    await user.click(
      screen.getByRole("button", { name: "Approve installation" }),
    );
    await waitFor(() =>
      expect(mocked.api).toHaveBeenCalledWith("install_dependencies", {
        consent: true,
      }),
    );
  });
  it("inspects an existing installation without importing until consent", async () => {
    const user = userEvent.setup();
    render(
      <Wizard
        {...props()}
        inspection={{
          ...ready,
          remotes: [
            {
              name: "gdrive",
              backend: "drive",
              configPath: "/tmp/rclone.conf",
            },
          ],
          mounts: [
            {
              source: "gdrive:",
              target: "/home/test/GoogleDrive",
              service: "rclone-gdrive.service",
            },
          ],
        }}
      />,
    );
    expect(
      screen.getByText("Google Drive is already connected"),
    ).toBeInTheDocument();
    await user.click(
      screen.getByRole("button", { name: /manage existing connection/i }),
    );
    expect(screen.getByRole("dialog")).toBeInTheDocument();
    expect(mocked.api).not.toHaveBeenCalledWith(
      "import_remote",
      expect.anything(),
    );
    await user.click(screen.getByRole("button", { name: "Cancel" }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });
  it("resumes authorized drafts at the folder step", async () => {
    const user = userEvent.setup();
    render(<Wizard {...props()} initialId="draft" />);
    expect(
      screen.getByRole("heading", { name: /where should your drive/i }),
    ).toBeInTheDocument();
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Continue" })).toBeEnabled(),
    );
    await user.click(screen.getByRole("button", { name: "Continue" }));
    await user.click(
      screen.getByRole("switch", { name: "Connect when I sign in" }),
    );
    expect(useSettings.getState().preferences.autostart).toBe(false);
    await user.click(screen.getByRole("button", { name: "Back" }));
    expect(useSettings.getState().mountPath).toBe("/home/test/GoogleDrive");
  });
  it("surfaces OAuth validation without starting authorization", async () => {
    const user = userEvent.setup();
    const p = props();
    render(<Wizard {...p} />);
    await user.click(screen.getByRole("button", { name: /get started/i }));
    await user.click(screen.getByRole("button", { name: "Continue" }));
    await user.click(
      screen.getByRole("button", { name: "Connect with Google" }),
    );
    expect(p.onError).toHaveBeenCalledWith(
      expect.objectContaining({ code: "oauth_credentials" }),
    );
    expect(mocked.api).not.toHaveBeenCalledWith(
      "begin_auth",
      expect.anything(),
    );
  });
});
