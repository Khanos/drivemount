import { useCallback, useEffect, useState } from "react";
import {
  CircleHelp,
  Cloud,
  LayoutDashboard,
  Leaf,
  Monitor,
  Moon,
  Plus,
  Settings2,
  Sun,
} from "lucide-react";
import { api, asError, isMock } from "./lib/api";
import type { AppError, Inspection } from "./lib/types";
import { useSettings } from "./lib/store";
import { Button } from "./components/ui/button";
import { Brand, ErrorRecoveryDialog, Spinner } from "./components/common";
import { Wizard } from "./components/Wizard";
import { DriveDashboard } from "./components/DriveDashboard";
export default function App() {
  const [inspection, setInspection] = useState<Inspection | null>(null);
  const [error, setError] = useState<AppError | null>(null);
  const [view, setView] = useState<"wizard" | "dashboard">("wizard");
  const [tab, setTab] = useState<"overview" | "settings" | "help">("overview");
  const [resumeId, setResumeId] = useState<string | null>(null);
  const [boot, setBoot] = useState(true);
  const { theme, setTheme, select } = useSettings();
  const reportError = useCallback((e: AppError) => setError(e), []);
  const refresh = useCallback(async () => {
    const result = await api("inspect_system");
    setInspection(result);
    const store = useSettings.getState();
    if (!store.mountPath) store.setMountPath(result.defaultMount);
    return result;
  }, []);
  useEffect(() => {
    let live = true;
    void refresh()
      .then((result) => {
        if (live && result.connections.some((c) => c.configured))
          setView("dashboard");
      })
      .catch((e) => {
        if (live) setError(asError(e));
      })
      .finally(() => {
        if (live) setBoot(false);
      });
    return () => {
      live = false;
    };
  }, [refresh]);
  useEffect(() => {
    const query = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = () =>
      document.documentElement.setAttribute(
        "data-theme",
        theme === "system" ? (query.matches ? "dark" : "light") : theme,
      );
    apply();
    query.addEventListener("change", apply);
    return () => query.removeEventListener("change", apply);
  }, [theme]);
  const add = () => {
    setResumeId(null);
    setView("wizard");
    setError(null);
  };
  const dashboard = () => {
    setView("dashboard");
    setTab("overview");
    setError(null);
  };
  return (
    <div className="app-shell">
      {isMock && (
        <div className="mock-banner">
          UI PREVIEW · Mock backend · No real system changes
        </div>
      )}
      <aside className="sidebar">
        <Brand />
        <div className="sidebar-content">
          {view === "dashboard" ? (
            <>
              <span className="nav-label">YOUR WORKSPACE</span>
              <nav aria-label="Main navigation">
                {[
                  { id: "overview", label: "Overview", icon: LayoutDashboard },
                  { id: "settings", label: "Settings", icon: Settings2 },
                  { id: "help", label: "Troubleshooting", icon: CircleHelp },
                ].map(({ id, label, icon: Icon }) => (
                  <button
                    key={id}
                    className={tab === id ? "active" : ""}
                    onClick={() => setTab(id as typeof tab)}
                  >
                    <Icon size={18} />
                    {label}
                  </button>
                ))}
              </nav>
              <Button variant="ghost" className="add-connection" onClick={add}>
                <Plus size={16} />
                Add a connection
              </Button>
            </>
          ) : (
            <div className="setup-sidebar">
              <span className="nav-label">A PLACE FOR YOUR CLOUD</span>
              <h2>
                Less setup.
                <br />
                More <em>at home.</em>
              </h2>
              <p>
                A familiar folder.
                <br />A little everyday magic.
              </p>
              <div className="sidebar-line" />
              <span>
                <Cloud size={17} />
                Google Drive, connected
              </span>
              <span>
                <Monitor size={17} />
                Your Linux desktop
              </span>
              <span>
                <Leaf size={17} />
                Light on your system
              </span>
              {!!inspection?.connections.length && (
                <Button variant="secondary" size="small" onClick={dashboard}>
                  View dashboard
                </Button>
              )}
            </div>
          )}
        </div>
        <div className="sidebar-bottom">
          <div className="theme-picker" role="group" aria-label="Color theme">
            {[
              { id: "light", icon: Sun, label: "Light theme" },
              { id: "system", icon: Monitor, label: "System theme" },
              { id: "dark", icon: Moon, label: "Dark theme" },
            ].map(({ id, icon: Icon, label }) => (
              <button
                key={id}
                aria-label={label}
                aria-pressed={theme === id}
                onClick={() => setTheme(id as typeof theme)}
              >
                <Icon size={16} />
              </button>
            ))}
          </div>
          <p>
            Independent by design.
            <br />
            Powered by rclone. Made for Linux.
          </p>
          <span className="version">v0.1.0</span>
        </div>
      </aside>
      <main className="main-area">
        <header className="topbar">
          <span>
            {view === "wizard"
              ? "WELCOME TO DRIVEMOUNT"
              : tab === "overview"
                ? "YOUR CONNECTION"
                : tab === "settings"
                  ? "PREFERENCES"
                  : "HELP & DETAILS"}
          </span>
          <span className="linux-tag">
            <span />
            Built for Linux
          </span>
        </header>
        <div className="global-error">
          <ErrorRecoveryDialog
            error={error}
            onDismiss={() => setError(null)}
            onRetry={
              error?.code === "desktop_required"
                ? undefined
                : () =>
                    void refresh()
                      .then(() => setError(null))
                      .catch((e) => setError(asError(e)))
            }
          />
        </div>
        {boot ? (
          <div className="boot-state">
            <div className="skeleton skeleton-heading" />
            <div className="skeleton skeleton-card" />
            <Spinner label="Getting to know your computer…" />
          </div>
        ) : inspection ? (
          view === "wizard" ? (
            <Wizard
              key={resumeId ?? "new"}
              inspection={inspection}
              onRefresh={refresh}
              onDashboard={dashboard}
              onError={reportError}
              initialId={resumeId}
            />
          ) : (
            <DriveDashboard
              inspection={inspection}
              onRefresh={refresh}
              onAdd={add}
              onResume={(id) => {
                select(id);
                setResumeId(id);
                const c = inspection.connections.find((c) => c.id === id);
                if (c) {
                  useSettings.getState().setMountPath(c.mountPath);
                  useSettings.getState().setPreferences(c.preferences);
                }
                setView("wizard");
              }}
              onError={reportError}
              tab={tab}
            />
          )
        ) : (
          <div className="empty-state">
            <Cloud size={48} />
            <h1>Your Drive starts here.</h1>
            <p>
              Open the desktop application to check and connect this computer.
            </p>
            <Button
              onClick={() => {
                setBoot(true);
                void refresh()
                  .catch((e) => setError(asError(e)))
                  .finally(() => setBoot(false));
              }}
            >
              Retry system check
            </Button>
          </div>
        )}
        <footer className="app-footer">
          <span>Your Google Drive, right in your Linux files.</span>
          <span>Connected storage. Familiar feeling.</span>
        </footer>
      </main>
    </div>
  );
}
