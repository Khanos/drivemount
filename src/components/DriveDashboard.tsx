import {
  Activity,
  ArrowRight,
  Check,
  ClipboardCopy,
  Cloud,
  ExternalLink,
  FolderOpen,
  HardDrive,
  Info,
  KeyRound,
  Link2,
  Power,
  RefreshCw,
  Settings2,
  ShieldCheck,
  Unplug,
} from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { api, asError } from "../lib/api";
import { useSettings } from "../lib/store";
import type {
  AppError,
  Connection,
  ConnectionStatus,
  Inspection,
  Preferences,
} from "../lib/types";
import { Button } from "./ui/button";
import { ConfirmationDialog } from "./ui/dialog";
import { Notice, PreferenceToggle, Spinner, StatusIndicator } from "./common";
import { ConnectionPreferences } from "./Preferences";
import { OAuthConnectionCard } from "./OAuthConnectionCard";
export function DiagnosticsPanel({
  id,
  onError,
}: {
  id: string;
  onError: (e: AppError) => void;
}) {
  const [report, setReport] = useState("");
  const [details, setDetails] = useState("");
  const [busy, setBusy] = useState(false);
  const [copied, setCopied] = useState(false);
  const refresh = useCallback(async () => {
    setBusy(true);
    try {
      const diagnostics = await api("read_diagnostics", { id });
      setReport(diagnostics.report);
      setDetails(diagnostics.details);
      setCopied(false);
    } catch (e) {
      onError(asError(e));
    } finally {
      setBusy(false);
    }
  }, [id, onError]);
  useEffect(() => {
    void refresh();
  }, [refresh]);
  return (
    <section className="diagnostics">
      <div className="inline-heading">
        <Activity size={19} />
        <h3>Connection diagnostics</h3>
      </div>
      <p>The shareable report contains connection status only.</p>
      <div className="row">
        <Button
          variant="secondary"
          size="small"
          onClick={() => void refresh()}
          disabled={busy}
        >
          <RefreshCw size={15} />
          Refresh diagnostics
        </Button>
        <Button
          variant="ghost"
          size="small"
          disabled={!report || busy}
          onClick={() =>
            void navigator.clipboard
              .writeText(report)
              .then(() => setCopied(true))
              .catch(() =>
                onError({
                  code: "clipboard",
                  message:
                    "Clipboard access is unavailable. Select the diagnostic text and copy it with your keyboard.",
                  recoverable: true,
                }),
              )
          }
        >
          {copied ? <Check size={15} /> : <ClipboardCopy size={15} />}{" "}
          {copied ? "Copied" : "Copy shareable report"}
        </Button>
      </div>
      {busy ? (
        <Spinner label="Reading connection events…" />
      ) : (
        <>
          <details open>
            <summary>Shareable status report</summary>
            <textarea
              aria-label="Redacted diagnostic report"
              readOnly
              value={report}
              rows={12}
            />
          </details>
          <details>
            <summary>Advanced local connection events</summary>
            <p>
              Credentials and common personal details are filtered. Messages may
              still contain private file names or other information. Review them
              yourself before sharing. These events are excluded from the copy
              button.
            </p>
            <textarea
              aria-label="Local connection events"
              readOnly
              value={details}
              rows={12}
            />
          </details>
        </>
      )}
    </section>
  );
}
export function SettingsPanel({
  connection,
  freeBytes,
  onSave,
  busy,
}: {
  connection: Connection;
  freeBytes: number;
  onSave: (path: string, p: Preferences) => void;
  busy: boolean;
}) {
  const [p, setP] = useState(connection.preferences);
  const [path, setPath] = useState(connection.mountPath);
  const [valid, setValid] = useState(true);
  useEffect(() => {
    setP(connection.preferences);
    setPath(connection.mountPath);
  }, [connection]);
  const validPreferences =
    p.cache.sizeGib >= 1 &&
    p.cache.sizeGib <= 1024 &&
    p.cache.ageHours >= 1 &&
    p.cache.ageHours <= 8760;
  if (!connection.managed)
    return (
      <Notice>
        This connection uses an existing service created outside DriveMount. Its
        account, mount location, and cache settings are preserved. You can
        manage its connection state and automatic startup from Overview.
      </Notice>
    );
  return (
    <fieldset disabled={busy} className="settings-panel">
      <Notice>
        Applying changes disconnects and reconnects Drive. Close open files
        first; pending uploads must finish before the connection can stop.
      </Notice>
      <div className="field-section">
        <label htmlFor="settings-path">Drive folder</label>
        <input
          id="settings-path"
          value={path}
          disabled={busy}
          onChange={(e) => {
            setPath(e.target.value);
            setValid(
              e.target.value.startsWith("/") && !e.target.value.includes(".."),
            );
          }}
        />
        <p className="help">
          The folder must be empty and owned by your account. It will be checked
          again before connecting.
        </p>
      </div>
      <ConnectionPreferences value={p} onChange={setP} freeBytes={freeBytes} />
      <Button
        disabled={busy || !valid || !validPreferences}
        onClick={() => onSave(path, p)}
      >
        <Settings2 size={16} />
        Review and apply changes
      </Button>
    </fieldset>
  );
}
export function ConnectionStatusCard({
  connection,
  status,
  busy,
  onOpen,
  onAction,
}: {
  connection: Connection;
  status: ConnectionStatus | null;
  busy: boolean;
  onOpen: () => void;
  onAction: (a: "start" | "stop" | "restart") => void;
}) {
  return (
    <section className="connection-card">
      <div className="connection-card-top">
        <div className="drive-emblem">
          <Cloud size={31} strokeWidth={1.5} />
        </div>
        <div>
          <h2>Google Drive</h2>
          <p>{connection.remote}</p>
        </div>
        {status ? (
          <StatusIndicator state={status.state} />
        ) : (
          <Spinner label="Checking…" />
        )}
      </div>
      <div className="mount-location">
        <span>YOUR DRIVE FOLDER</span>
        <strong>
          <FolderOpen size={20} />
          {connection.mountPath}
        </strong>
      </div>
      <p className="connection-detail" aria-live="polite">
        {status?.detail ?? "Checking your service and actual filesystem mount…"}
      </p>
      <div className="connection-actions">
        <Button onClick={onOpen} disabled={!status?.mounted || busy}>
          <FolderOpen size={17} />
          Open Drive
          <ExternalLink size={14} />
        </Button>
        {status?.mounted ? (
          <>
            <Button
              variant="secondary"
              onClick={() => onAction("stop")}
              disabled={busy}
            >
              <Unplug size={16} />
              Disconnect
            </Button>
            <Button
              variant="ghost"
              aria-label="Restart connection"
              onClick={() => onAction("restart")}
              disabled={busy}
            >
              <RefreshCw size={17} />
            </Button>
          </>
        ) : (
          <Button
            variant="secondary"
            onClick={() => onAction("start")}
            disabled={busy || !status}
          >
            <Link2 size={16} />
            Reconnect
          </Button>
        )}
      </div>
    </section>
  );
}
export function DriveDashboard({
  inspection,
  onRefresh,
  onAdd,
  onResume,
  onError,
  tab,
}: {
  inspection: Inspection;
  onRefresh: () => Promise<Inspection>;
  onAdd: () => void;
  onResume: (id: string) => void;
  onError: (e: AppError) => void;
  tab: "overview" | "settings" | "help";
}) {
  const { selectedId, select } = useSettings();
  const c =
    inspection.connections.find((c) => c.id === selectedId) ??
    inspection.connections.find((c) => c.configured) ??
    inspection.connections[0];
  const [s, setS] = useState<ConnectionStatus | null>(null);
  const [busy, setBusy] = useState(false);
  const [confirmation, setConfirmation] = useState<{
    action: "stop" | "restart";
    unknown: boolean;
  } | null>(null);
  const [settings, setSettings] = useState<{
    path: string;
    p: Preferences;
  } | null>(null);
  const [reauth, setReauth] = useState(false);
  const [events, setEvents] = useState<string[]>([]);
  const addEvent = (message: string) =>
    setEvents((old) =>
      [
        `${new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })} · ${message}`,
        ...old,
      ].slice(0, 5),
    );
  const statusId = c?.id;
  const configured = c?.configured;
  const statusGeneration = useRef<symbol | null>(null);
  const statusInFlight = useRef(false);
  const refresh = useCallback(async () => {
    if (!configured || !statusId || statusInFlight.current) return;
    const generation = Symbol();
    statusGeneration.current = generation;
    statusInFlight.current = true;
    try {
      const result = await api("connection_status", { id: statusId });
      if (generation === statusGeneration.current) setS(result);
    } catch (e) {
      if (generation === statusGeneration.current) onError(asError(e));
    } finally {
      if (generation === statusGeneration.current)
        statusInFlight.current = false;
    }
  }, [statusId, configured, onError]);
  useEffect(() => {
    setS(null);
    void refresh();
    const timer = setInterval(() => {
      if (document.visibilityState === "visible") void refresh();
    }, 30000);
    return () => {
      statusGeneration.current = null;
      statusInFlight.current = false;
      clearInterval(timer);
    };
  }, [refresh]);
  const run = async (work: () => Promise<void>) => {
    setBusy(true);
    try {
      await work();
    } catch (e) {
      onError(asError(e));
    } finally {
      setBusy(false);
    }
  };
  const action = async (
    a: "start" | "stop" | "restart",
    acknowledgeUnknown = false,
  ) => {
    if (!c) return;
    statusGeneration.current = Symbol();
    statusInFlight.current = false;
    setBusy(true);
    try {
      setS(
        await api("connection_action", {
          id: c.id,
          action: a,
          acknowledgeUnknown,
        }),
      );
      setConfirmation(null);
      addEvent(
        a === "stop" ? "Drive disconnected" : "Drive connection refreshed",
      );
    } catch (e) {
      const error = asError(e);
      if (error.code === "pending_unknown" && a !== "start") {
        setConfirmation({ action: a, unknown: true });
      } else {
        onError(error);
        setConfirmation(null);
      }
    } finally {
      setBusy(false);
    }
  };
  const autostart = async (enabled: boolean) => {
    await run(async () => {
      await api("set_autostart", { id: c!.id, enabled });
      setS((old) => (old ? { ...old, autostart: enabled } : old));
      await onRefresh();
      addEvent(
        enabled
          ? "Automatic connection enabled"
          : "Automatic connection disabled",
      );
    });
  };
  const onReauthorized = useCallback(() => {
    setReauth(false);
    void refresh();
  }, [refresh]);
  if (!c)
    return (
      <div className="empty-state">
        <Cloud size={48} />
        <h1>A home for your Google Drive.</h1>
        <p>Connect an account to see it here.</p>
        <Button onClick={onAdd}>
          Set up a connection
          <ArrowRight size={16} />
        </Button>
      </div>
    );
  if (!c.configured)
    return (
      <div className="empty-state">
        <Cloud size={48} />
        <h1>Let’s finish connecting.</h1>
        <p>
          We found an unfinished setup. Your saved account configuration is
          preserved.
        </p>
        <Button onClick={() => onResume(c.id)} disabled={!c.authorized}>
          Continue setup
          <ArrowRight size={16} />
        </Button>
        {!c.authorized && (
          <Button variant="secondary" onClick={() => setReauth(true)}>
            Connect Google account
          </Button>
        )}
        {inspection.connections.some((other) => other.configured) && (
          <Button
            variant="ghost"
            onClick={() =>
              select(
                inspection.connections.find((other) => other.configured)!.id,
              )
            }
          >
            Back to connected Drive
          </Button>
        )}
        <ConfirmationDialog
          open={reauth}
          onOpenChange={setReauth}
          title="Connect your Google account"
          description="Finish authorization in your normal browser."
        >
          <OAuthConnectionCard
            appOAuth={inspection.appOauthAvailable}
            connectionId={c.id}
            onConnected={() => {
              setReauth(false);
              void onRefresh();
            }}
            onError={onError}
          />
        </ConfirmationDialog>
      </div>
    );
  return (
    <div className="dashboard">
      <div className="dashboard-heading">
        <div>
          <span className="eyebrow">
            {tab === "overview"
              ? "A little less distance."
              : tab === "settings"
                ? "Just the way you like it."
                : "Here to help."}
          </span>
          <h1>
            {tab === "overview"
              ? "Your Drive, at home."
              : tab === "settings"
                ? "Connection settings."
                : "Let’s keep things flowing."}
          </h1>
          <p>
            {tab === "overview"
              ? "Your cloud files, one familiar folder away."
              : tab === "settings"
                ? "Fine-tune how Drive connects to your computer."
                : "Understand your connection and find a way forward."}
          </p>
        </div>
        <Button
          variant="ghost"
          size="small"
          disabled={busy}
          onClick={() =>
            void run(async () => {
              await onRefresh();
              await refresh();
            })
          }
          aria-label="Refresh connection status"
        >
          <RefreshCw size={17} />
        </Button>
      </div>
      {inspection.connections.length > 1 && (
        <label className="connection-select">
          Connection
          <select
            disabled={busy}
            value={c.id}
            onChange={(e) => select(e.target.value)}
          >
            {inspection.connections.map((c) => (
              <option key={c.id} value={c.id}>
                {c.remote}
                {!c.configured ? " · setup unfinished" : ""}
              </option>
            ))}
          </select>
        </label>
      )}
      {tab === "overview" && (
        <>
          <ConnectionStatusCard
            connection={c}
            status={s}
            busy={busy}
            onOpen={() => void run(() => api("open_drive", { id: c.id }))}
            onAction={(a) =>
              a === "start"
                ? void action(a)
                : setConfirmation({ action: a, unknown: false })
            }
          />
          <div className="dashboard-grid">
            <div className="info-card">
              <Power size={20} />
              <span>READY WHEN YOU ARE</span>
              <h3>Connect on sign-in</h3>
              <PreferenceToggle
                id="dashboard-autostart"
                title={
                  s?.autostart
                    ? "Automatic connection is on"
                    : "Connect automatically"
                }
                description="Available when your desktop starts."
                checked={s?.autostart ?? false}
                disabled={busy || !s}
                onChange={(v) => void autostart(v)}
              />
            </div>
            <div className="info-card">
              <HardDrive size={20} />
              <span>A LITTLE LOCAL SPACE</span>
              <h3>
                {c.managed
                  ? `${c.preferences.cache.sizeGib} GiB cache target`
                  : "Existing cache settings"}
              </h3>
              <p>
                {c.managed
                  ? `${s?.cacheBytes === null || s?.cacheBytes === undefined ? "Usage unavailable" : `${Math.round(s.cacheBytes / 1024 ** 2)} MiB currently cached`}. Files are fetched as needed.`
                  : "Your original cache configuration is preserved."}
              </p>
              <small>
                {s?.pendingUploads === null || s?.pendingUploads === undefined
                  ? "Upload state unavailable"
                  : s.pendingUploads > 0
                    ? `${s.pendingUploads} uploads need to finish`
                    : "No pending uploads reported"}
              </small>
            </div>
          </div>
          <div className="quiet-card">
            <ShieldCheck size={22} />
            <div>
              <strong>Still your files. Still your Google Drive.</strong>
              <p>
                This folder provides cloud access. It isn’t a full offline copy
                or backup.
              </p>
            </div>
          </div>
          {events.length > 0 && (
            <div className="recent-events">
              <h3>Recent activity</h3>
              {events.map((e, i) => (
                <p key={`${e}-${i}`}>{e}</p>
              ))}
            </div>
          )}
        </>
      )}
      {tab === "settings" && (
        <>
          <SettingsPanel
            connection={c}
            freeBytes={inspection.freeCacheBytes}
            busy={busy}
            onSave={(path, p) => setSettings({ path, p })}
          />
          <div className="field-section">
            <h3>Google account</h3>
            <p className="help">
              Renew authorization if Google asks you to sign in again.
              Disconnect Drive first.
            </p>
            <Button
              variant="secondary"
              disabled={busy || !c.ownedRemote || s?.mounted}
              onClick={() => setReauth(true)}
            >
              <KeyRound size={16} />
              Renew Google connection
            </Button>
            {!c.ownedRemote && (
              <p className="help">
                This account was created outside DriveMount. Renew it using its
                original configuration tool.
              </p>
            )}
          </div>
        </>
      )}
      {tab === "help" && (
        <>
          <Notice>
            Close files before disconnecting. If uploads are still pending, let
            them finish. DriveMount never clears cached edits automatically.
          </Notice>
          <DiagnosticsPanel id={c.id} onError={onError} />
          <details className="connection-details">
            <summary>
              <Info size={16} />
              Connection details
            </summary>
            <dl>
              {[
                ["Remote", c.remote],
                ["Service", c.service],
                ["Configuration", c.configPath],
                [
                  "Service ownership",
                  c.managed ? "DriveMount" : "Existing setup",
                ],
                [
                  "Credentials ownership",
                  c.ownedRemote ? "DriveMount" : "Existing setup",
                ],
                ["Application version", "0.1.0"],
              ].map(([k, v]) => (
                <div key={k}>
                  <dt>{k}</dt>
                  <dd>{v}</dd>
                </div>
              ))}
            </dl>
          </details>
        </>
      )}
      <ConfirmationDialog
        open={!!confirmation}
        onOpenChange={(open) => {
          if (!open && !busy) setConfirmation(null);
        }}
        title={
          confirmation?.unknown
            ? "Upload state cannot be confirmed"
            : confirmation?.action === "restart"
              ? "Restart your Drive connection?"
              : "Disconnect Google Drive?"
        }
        description={
          confirmation?.unknown
            ? "Close all Drive files and check that your edits have uploaded. Continuing may interrupt pending changes. Cached data will be preserved."
            : "Close any open Drive files first. DriveMount will check for open files and unfinished uploads before stopping the connection."
        }
      >
        <div className="dialog-actions">
          <Button
            variant="secondary"
            onClick={() => setConfirmation(null)}
            disabled={busy}
          >
            Keep connected
          </Button>
          <Button
            variant={confirmation?.unknown ? "danger" : "default"}
            disabled={busy}
            onClick={() =>
              confirmation &&
              void action(confirmation.action, confirmation.unknown)
            }
          >
            {busy ? (
              <Spinner />
            ) : confirmation?.unknown ? (
              "I understand, continue"
            ) : (
              "Continue"
            )}
          </Button>
        </div>
      </ConfirmationDialog>
      <ConfirmationDialog
        open={!!settings}
        onOpenChange={(open) => {
          if (!open && !busy) setSettings(null);
        }}
        title="Apply connection settings?"
        description="Drive will disconnect, validate the new folder and settings, then reconnect. Any unfinished uploads must complete first."
      >
        <div className="dialog-actions">
          <Button
            variant="secondary"
            disabled={busy}
            onClick={() => setSettings(null)}
          >
            Cancel
          </Button>
          <Button
            disabled={busy}
            onClick={() =>
              void run(async () => {
                if (!settings) return;
                if (s?.mounted) {
                  await api("connection_action", {
                    id: c.id,
                    action: "stop",
                    acknowledgeUnknown: false,
                  });
                }
                await api("connect_drive", {
                  request: {
                    id: c.id,
                    mountPath: settings.path,
                    preferences: settings.p,
                  },
                });
                setSettings(null);
                await onRefresh();
                await refresh();
                addEvent("Connection settings updated");
              })
            }
          >
            {busy ? <Spinner /> : "Apply changes"}
          </Button>
        </div>
      </ConfirmationDialog>
      <ConfirmationDialog
        open={reauth}
        onOpenChange={setReauth}
        title="Renew your Google connection"
        description="Sign in with Google in your normal browser. Your existing Drive files stay in place."
      >
        <OAuthConnectionCard
          appOAuth={true}
          reauth
          connectionId={c.id}
          onConnected={onReauthorized}
          onError={onError}
        />
      </ConfirmationDialog>
    </div>
  );
}
