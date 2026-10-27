import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { DriveDashboard } from "./DriveDashboard";
import { defaults, useSettings } from "../lib/store";
import type { Connection, Inspection } from "../lib/types";
vi.mock("../lib/api", () => ({
  api: vi.fn().mockResolvedValue({
    id: "configured",
    state: "connected",
    mounted: true,
    activeState: "active",
    autostart: true,
    pendingUploads: 0,
    openFiles: 0,
    cacheBytes: 0,
    detail: "Mounted",
  }),
  asError: (e: unknown) => e,
}));
afterEach(cleanup);
it("reopens a configured connection when an earlier unfinished draft exists", async () => {
  const base: Connection = {
    id: "draft",
    remote: "draft_account",
    configPath: "/tmp/config",
    mountPath: "/home/demo/unfinished",
    service: "draft.service",
    cachePath: "/tmp/cache",
    socketPath: "/tmp/socket",
    managed: true,
    ownedRemote: true,
    authorized: false,
    configured: false,
    preferences: structuredClone(defaults),
  };
  const inspection: Inspection = {
    distro: "Linux",
    packageManager: "apt",
    packages: [],
    checks: [],
    remotes: [],
    mounts: [],
    services: [],
    connections: [
      base,
      {
        ...base,
        id: "configured",
        remote: "ready_account",
        mountPath: "/home/demo/Connected Drive",
        authorized: true,
        configured: true,
      },
    ],
    defaultMount: "/home/demo/GoogleDrive",
    appOauthAvailable: false,
    freeCacheBytes: 50 * 1024 ** 3,
  };
  useSettings.setState({ selectedId: null });
  render(
    <DriveDashboard
      inspection={inspection}
      onRefresh={vi.fn().mockResolvedValue(inspection)}
      onAdd={vi.fn()}
      onResume={vi.fn()}
      onError={vi.fn()}
      tab="overview"
    />,
  );
  expect(
    await screen.findByRole("heading", { name: "Google Drive" }),
  ).toBeInTheDocument();
  expect(screen.getByText("/home/demo/Connected Drive")).toBeInTheDocument();
  expect(
    screen.queryByRole("heading", { name: "Let’s finish connecting." }),
  ).not.toBeInTheDocument();
});
