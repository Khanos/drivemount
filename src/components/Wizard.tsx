import {
  ArrowLeft,
  ArrowRight,
  Check,
  ChevronRight,
  Cloud,
  CloudDownload,
  Folder,
  FolderOpen,
  Heart,
  LockKeyhole,
  Monitor,
  ShieldCheck,
} from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { api, asError } from "../lib/api";
import { useSettings } from "../lib/store";
import type {
  AppError,
  ConnectionStatus,
  Inspection,
  Progress,
} from "../lib/types";
import { Button } from "./ui/button";
import { ConfirmationDialog } from "./ui/dialog";
import { Checkmark, Notice, Spinner } from "./common";
import { SystemCheck } from "./SystemCheck";
import { OAuthConnectionCard } from "./OAuthConnectionCard";
import { FolderPicker } from "./FolderPicker";
import { ConnectionPreferences } from "./Preferences";
const steps = [
  "Welcome",
  "System check",
  "Google account",
  "Drive location",
  "Preferences",
  "Review",
  "Ready",
];
export function WizardProgress({ step }: { step: number }) {
  return (
    <ol className="wizard-progress" aria-label="Setup progress">
      {steps.slice(1, 6).map((label, i) => (
        <li
          key={label}
          aria-current={step === i + 1 ? "step" : undefined}
          className={step > i + 1 ? "done" : step === i + 1 ? "current" : ""}
        >
          <span>{step > i + 1 ? <Check size={13} /> : i + 1}</span>
          <small>{label}</small>
        </li>
      ))}
    </ol>
  );
}
export function WelcomeScreen({
  inspection,
  onStart,
  onManage,
  onImport,
  busy,
}: {
  inspection: Inspection;
  onStart: () => void;
  onManage: () => void;
  onImport: (name: string) => void;
  busy: boolean;
}) {
  const existing = inspection.mounts.length > 0;
  return (
    <div className="welcome">
      <span className="welcome-badge">
        <span />A little closer to your cloud
      </span>
      <h1>
        Bring Google Drive
        <br />
        to <span>Linux.</span>
      </h1>
      <p className="welcome-description">
        Your Google Drive, right in your Linux files.
        <br />
        Connect once. Feel right at home.
      </p>
      <div className="welcome-illustration" aria-hidden="true">
        <div className="orbit orbit-one" />
        <div className="orbit orbit-two" />
        <div className="art-file file-one">
          <div />
          <div />
          <div />
        </div>
        <div className="art-file file-two">
          <div />
          <div />
          <div />
        </div>
        <div className="cloud-bubble">
          <Cloud size={39} strokeWidth={1.3} />
        </div>
        <div className="hero-folder">
          <Folder size={118} strokeWidth={0.8} />
          <span>
            <Check size={20} />
          </span>
        </div>
        <span className="art-spark spark-one">✦</span>
        <span className="art-spark spark-two">✧</span>
        <div className="art-caption">
          <span />
          Made for your Linux desktop
        </div>
      </div>
      <div className="welcome-features">
        <span>
          <FolderOpen size={16} />
          Your usual file manager
        </span>
        <span>
          <LockKeyhole size={16} />
          Secure Google sign-in
        </span>
        <span>
          <Heart size={16} />
          No terminal needed
        </span>
      </div>
      {existing && (
        <div className="existing-card">
          <ShieldCheck size={25} />
          <div>
            <strong>Google Drive is already connected</strong>
            <p>
              We found your existing setup. Your files and credentials will be
              preserved.
            </p>
          </div>
          <Button
            variant="secondary"
            size="small"
            disabled={busy}
            onClick={() =>
              inspection.connections.some((c) => c.configured)
                ? onManage()
                : onImport(
                    inspection.remotes.find((r) =>
                      inspection.mounts.some((m) => m.source === `${r.name}:`),
                    )?.name ??
                      inspection.remotes[0]?.name ??
                      "gdrive",
                  )
            }
          >
            Manage existing connection
            <ChevronRight size={14} />
          </Button>
        </div>
      )}
      <Button className="get-started" onClick={onStart}>
        Get Started
        <ArrowRight size={17} />
      </Button>
      {inspection.connections.length > 0 && !existing && (
        <Button variant="ghost" onClick={onManage}>
          Manage existing connection
        </Button>
      )}
      <p className="welcome-footnote">
        Files load on demand. This is a connected folder,
        <br />
        not a complete offline backup or sync client.
      </p>
    </div>
  );
}
export function SetupSummary({
  path,
  preferences,
}: {
  path: string;
  preferences: ReturnType<typeof useSettings.getState>["preferences"];
}) {
  return (
    <div className="summary-card">
      {[
        ["Google account", "Connected securely"],
        ["Drive folder", path],
        ["File access", preferences.readOnly ? "Read-only" : "Read and write"],
        ["Connect on sign-in", preferences.autostart ? "Enabled" : "Disabled"],
        [
          "Temporary storage",
          `${preferences.cache.sizeGib} GiB target · ${preferences.cache.ageHours} hours`,
        ],
      ].map(([k, v]) => (
        <div key={k}>
          <span>{k}</span>
          <strong>{v}</strong>
        </div>
      ))}
    </div>
  );
}
export function InstallationProgress({
  progress,
}: {
  progress: Progress | null;
}) {
  return (
    <div className="installation-progress" role="status">
      <Spinner label={progress?.message || "Preparing your connection…"} />
      <p>
        Your folder is ready when the connection and mount checks both pass.
      </p>
    </div>
  );
}
export function SuccessScreen({
  path,
  onOpen,
  onDashboard,
  busy,
}: {
  path: string;
  onOpen: () => void;
  onDashboard: () => void;
  busy: boolean;
}) {
  return (
    <div className="success-screen">
      <div className="success-art">
        <FolderOpen size={62} strokeWidth={1.2} />
        <span>
          <Check size={22} />
        </span>
      </div>
      <span className="eyebrow">Connected. And right at home.</span>
      <h1>Your Google Drive is ready!</h1>
      <p>Your files are waiting in your usual file manager.</p>
      <code className="path-pill">{path}</code>
      <Button onClick={onOpen} disabled={busy}>
        <FolderOpen size={18} />
        Open Google Drive
      </Button>
      <Button variant="ghost" onClick={onDashboard}>
        Go to Dashboard
        <ArrowRight size={16} />
      </Button>
      <p className="help">
        Files load from the cloud as needed.
        <br />
        Keep an internet connection for files that aren’t cached.
      </p>
    </div>
  );
}
export function Wizard({
  inspection,
  onRefresh,
  onDashboard,
  onError,
  initialId,
}: {
  inspection: Inspection;
  onRefresh: () => Promise<Inspection>;
  onDashboard: () => void;
  onError: (e: AppError) => void;
  initialId: string | null;
}) {
  const [step, setStep] = useState(initialId ? 3 : 0);
  const [busy, setBusy] = useState(false);
  const [id, setId] = useState<string | null>(initialId);
  const [pathValid, setPathValid] = useState(false);
  const [progress, setProgress] = useState<Progress | null>(null);
  const [importName, setImportName] = useState<string | null>(null);
  const [success, setSuccess] = useState<ConnectionStatus | null>(null);
  const { mountPath, preferences, setMountPath, setPreferences, select } =
    useSettings();
  const onConnected = useCallback((id: string) => {
    setId(id);
  }, []);
  const mutate = async (work: () => Promise<void>) => {
    setBusy(true);
    try {
      await work();
    } catch (e) {
      onError(asError(e));
    } finally {
      setBusy(false);
    }
  };
  useEffect(() => {
    if (step !== 5 || !busy) return;
    let live = true;
    const timer = setInterval(
      () =>
        void api("setup_progress")
          .then((p) => {
            if (live) setProgress(p);
          })
          .catch(() => {}),
      500,
    );
    return () => {
      live = false;
      clearInterval(timer);
    };
  }, [step, busy]);
  const canContinue =
    step === 1
      ? inspection.checks.every((c) => c.state === "ready")
      : step === 2
        ? !!id
        : step === 3
          ? pathValid
          : preferences.cache.sizeGib >= 1 &&
            preferences.cache.sizeGib <= 1024 &&
            preferences.cache.ageHours >= 1 &&
            preferences.cache.ageHours <= 8760;
  async function importExisting() {
    if (!importName) return;
    await mutate(async () => {
      const c = await api("import_remote", { name: importName, consent: true });
      setId(c.id);
      select(c.id);
      setMountPath(c.mountPath);
      setPreferences(c.preferences);
      setImportName(null);
      await onRefresh();
      if (c.configured) onDashboard();
      else setStep(3);
    });
  }
  async function connect() {
    await mutate(async () => {
      const s = await api("connect_drive", {
        request: { id: id!, mountPath, preferences },
      });
      if (!s.mounted)
        throw {
          code: "mount_failed",
          message:
            "Drive is not mounted yet. Refresh diagnostics and try again.",
        };
      setSuccess(s);
      select(id);
      await onRefresh();
      setStep(6);
    });
  }
  if (step === 0)
    return (
      <>
        <WelcomeScreen
          inspection={inspection}
          onStart={() => setStep(1)}
          onManage={onDashboard}
          onImport={setImportName}
          busy={busy}
        />
        <ImportDialog
          name={importName}
          busy={busy}
          onClose={() => setImportName(null)}
          onImport={() => void importExisting()}
        />
      </>
    );
  return (
    <div className="wizard-layout">
      <WizardProgress step={step} />
      <div className={`wizard-body ${step === 6 ? "wizard-success" : ""}`}>
        {step === 1 && (
          <SystemCheck
            inspection={inspection}
            busy={busy}
            refresh={() =>
              void mutate(async () => {
                await onRefresh();
              })
            }
            install={() =>
              void mutate(async () => {
                await api("install_dependencies", { consent: true });
                await onRefresh();
              })
            }
          />
        )}
        {step === 2 && (
          <>
            <div className="section-heading">
              <span className="eyebrow">A secure introduction</span>
              <h1>Connect your Google account.</h1>
              <p>
                Your files stay in Google Drive. You choose how to access them.
              </p>
            </div>
            {id ? (
              <Notice tone="success">
                <Checkmark />
                Your Google account is connected. Continue to choose your
                folder.
              </Notice>
            ) : (
              <>
                {inspection.remotes.length > 0 && (
                  <div className="existing-remotes">
                    <h3>Use an existing Google connection</h3>
                    {inspection.remotes.map((r) => (
                      <Button
                        variant="secondary"
                        key={r.name}
                        onClick={() => setImportName(r.name)}
                      >
                        <Cloud size={17} />
                        {r.name}
                        <ChevronRight size={15} />
                      </Button>
                    ))}
                    <span className="divider-label">
                      or connect another account
                    </span>
                  </div>
                )}
                <OAuthConnectionCard
                  appOAuth={inspection.appOauthAvailable}
                  connectionId={null}
                  onConnected={onConnected}
                  onError={onError}
                />
              </>
            )}
          </>
        )}
        {step === 3 && (
          <>
            <div className="section-heading">
              <span className="eyebrow">A place to call home</span>
              <h1>Where should your Drive appear?</h1>
              <p>
                Choose where you’ll find your Google Drive in your file manager.
              </p>
            </div>
            <FolderPicker
              value={mountPath || inspection.defaultMount}
              onChange={setMountPath}
              onValidity={setPathValid}
              onError={onError}
            />
          </>
        )}
        {step === 4 && (
          <>
            <div className="section-heading">
              <span className="eyebrow">The little details</span>
              <h1>Make Drive work your way.</h1>
              <p>
                Our defaults work well for most people. You can change them
                later.
              </p>
            </div>
            <ConnectionPreferences
              value={preferences}
              onChange={setPreferences}
              freeBytes={inspection.freeCacheBytes}
            />
          </>
        )}
        {step === 5 && (
          <>
            <div className="section-heading">
              <span className="eyebrow">One last look</span>
              <h1>Everything is ready.</h1>
              <p>Here’s how Google Drive will connect to your computer.</p>
            </div>
            <SetupSummary path={mountPath} preferences={preferences} />
            {busy ? (
              <InstallationProgress progress={progress} />
            ) : (
              <Notice tone="success">
                <ShieldCheck size={18} />
                Your account is connected, your folder is chosen, and your files
                stay yours.
              </Notice>
            )}
            <div className="review-benefits">
              <span>
                <Monitor size={17} />
                Made for your desktop
              </span>
              <span>
                <CloudDownload size={17} />
                Files on demand
              </span>
            </div>
          </>
        )}
        {step === 6 && (
          <SuccessScreen
            path={mountPath}
            busy={busy}
            onOpen={() =>
              void mutate(() => api("open_drive", { id: success!.id }))
            }
            onDashboard={onDashboard}
          />
        )}
      </div>
      {step < 6 && (
        <footer className="wizard-footer">
          <Button
            variant="ghost"
            disabled={busy}
            onClick={() => {
              if (step === 2) void api("cancel_auth");
              setStep(step - 1);
            }}
          >
            <ArrowLeft size={16} />
            Back
          </Button>
          <span>Step {step} of 5</span>
          <Button
            disabled={busy || !canContinue}
            onClick={() => (step === 5 ? void connect() : setStep(step + 1))}
          >
            {busy ? (
              <Spinner label="Working…" />
            ) : step === 5 ? (
              <>
                <Cloud size={17} />
                Connect Drive
              </>
            ) : (
              <>
                Continue
                <ArrowRight size={16} />
              </>
            )}
          </Button>
        </footer>
      )}
      <ImportDialog
        name={importName}
        busy={busy}
        onClose={() => setImportName(null)}
        onImport={() => void importExisting()}
      />
    </div>
  );
}
function ImportDialog({
  name,
  onClose,
  onImport,
  busy,
}: {
  name: string | null;
  onClose: () => void;
  onImport: () => void;
  busy: boolean;
}) {
  return (
    <ConfirmationDialog
      open={!!name}
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
      title="Manage your existing connection?"
      description="DriveMount will remember this account and may start, stop, or enable its service when you ask. Existing credentials, service files, and cached data will be preserved."
    >
      <p className="import-remote">
        <Cloud size={20} />
        {name}
      </p>
      <Notice>
        Settings for a service created outside DriveMount remain controlled by
        its original setup. Disconnecting will ask for confirmation if pending
        uploads cannot be checked.
      </Notice>
      <div className="dialog-actions">
        <Button variant="secondary" onClick={onClose} disabled={busy}>
          Cancel
        </Button>
        <Button onClick={onImport} disabled={busy}>
          {busy ? <Spinner /> : "Manage existing connection"}
        </Button>
      </div>
    </ConfirmationDialog>
  );
}
