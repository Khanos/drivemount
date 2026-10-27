import {
  AlertCircle,
  ArrowUpRight,
  Check,
  Cloud,
  Folder,
  LoaderCircle,
  ShieldCheck,
  WifiOff,
} from "lucide-react";
import type { ReactNode } from "react";
import { Button } from "./ui/button";
import { Switch } from "./ui/switch";
import type { AppError } from "../lib/types";
export function Brand({ compact = false }: { compact?: boolean }) {
  return (
    <div className="brand">
      <div className="brand-icon">
        <Folder size={22} />
        <Cloud size={13} />
      </div>
      {!compact && (
        <span>
          DriveMount<span className="brand-dot">.</span>
        </span>
      )}
    </div>
  );
}
export function Spinner({ label = "Working…" }: { label?: string }) {
  return (
    <span className="loading" role="status">
      <LoaderCircle className="spin" size={18} />
      {label}
    </span>
  );
}
export function Notice({
  children,
  tone = "neutral",
}: {
  children: ReactNode;
  tone?: "neutral" | "success" | "warning";
}) {
  return (
    <div className={`notice notice-${tone}`}>
      {tone === "success" ? (
        <ShieldCheck size={18} />
      ) : tone === "warning" ? (
        <AlertCircle size={18} />
      ) : (
        <Cloud size={18} />
      )}
      <div>{children}</div>
    </div>
  );
}
export function ErrorRecoveryDialog({
  error,
  onDismiss,
  onRetry,
}: {
  error: AppError | null;
  onDismiss: () => void;
  onRetry?: () => void;
}) {
  if (!error) return null;
  return (
    <div className="error-box" role="alert">
      <AlertCircle size={20} />
      <div>
        <strong>
          {error.code === "authentication"
            ? "Reconnect your Google account"
            : error.code === "pending_writes"
              ? "Your changes need a little more time"
              : "Let’s get you connected"}
        </strong>
        <p>{error.message}</p>
        <div className="row">
          {onRetry && (
            <Button variant="secondary" size="small" onClick={onRetry}>
              Try again
            </Button>
          )}
          <Button variant="ghost" size="small" onClick={onDismiss}>
            Dismiss
          </Button>
        </div>
      </div>
    </div>
  );
}
export function PreferenceToggle({
  id,
  title,
  description,
  checked,
  onChange,
  disabled = false,
}: {
  id: string;
  title: string;
  description: string;
  checked: boolean;
  onChange: (v: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <div className="preference-toggle">
      <div>
        <label htmlFor={id}>{title}</label>
        <p id={`${id}-help`}>{description}</p>
      </div>
      <Switch
        id={id}
        checked={checked}
        onCheckedChange={onChange}
        disabled={disabled}
        aria-describedby={`${id}-help`}
      />
    </div>
  );
}
const labels: Record<string, string> = {
  connected: "Connected",
  connecting: "Connecting",
  reconnecting: "Reconnecting",
  disconnected: "Disconnected",
  offline: "Offline",
  authenticationRequired: "Authorization needed",
  error: "Needs attention",
};
export function StatusIndicator({ state }: { state: string }) {
  return (
    <span className={`status status-${state}`}>
      {state === "offline" ? (
        <WifiOff size={13} />
      ) : (
        <span className="status-dot" />
      )}
      {labels[state] ?? state}
    </span>
  );
}
export function Checkmark() {
  return (
    <span className="checkmark">
      <Check size={16} />
    </span>
  );
}
export function ExternalHelp({
  onClick,
  children,
}: {
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <button type="button" className="text-link" onClick={onClick}>
      {children}
      <ArrowUpRight size={14} />
    </button>
  );
}
