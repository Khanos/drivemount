import {
  Check,
  CircleAlert,
  Download,
  RefreshCw,
  ShieldCheck,
} from "lucide-react";
import { useState } from "react";
import type { Check as CheckType, Inspection } from "../lib/types";
import { Button } from "./ui/button";
import { ConfirmationDialog } from "./ui/dialog";
import { Notice, Spinner } from "./common";
export function DependencyItem({ check }: { check: CheckType }) {
  return (
    <div className="dependency-item">
      <span className={`dependency-icon dependency-${check.state}`}>
        {check.state === "ready" ? (
          <Check size={18} />
        ) : (
          <CircleAlert size={18} />
        )}
      </span>
      <div>
        <strong>{check.label}</strong>
        <p>{check.detail}</p>
      </div>
      <span className={`check-label ${check.state}`}>
        {
          {
            ready: "Ready",
            install: "Needs installation",
            attention: "Needs attention",
            unsupported: "Unsupported",
          }[check.state]
        }
      </span>
    </div>
  );
}
export function SystemCheck({
  inspection,
  busy,
  refresh,
  install,
}: {
  inspection: Inspection;
  busy: boolean;
  refresh: () => void;
  install: () => void;
}) {
  const [approve, setApprove] = useState(false);
  return (
    <>
      <div className="section-heading">
        <div>
          <span className="eyebrow">A quick check</span>
          <h1>A good home for your Drive.</h1>
          <p>We’ll make sure your computer has everything it needs.</p>
        </div>
        <Button
          variant="ghost"
          size="small"
          onClick={refresh}
          disabled={busy}
          aria-label="Refresh system check"
        >
          <RefreshCw size={16} />
        </Button>
      </div>
      <div className="check-list">
        {inspection.checks.map((c) => (
          <DependencyItem key={c.id} check={c} />
        ))}
      </div>
      {inspection.packages.length > 0 ? (
        <Notice>
          We need to install {inspection.packages.length} components. Your
          desktop will ask for administrator approval.
          <div className="package-list">
            {inspection.packages.map((p) => (
              <code key={p}>{p}</code>
            ))}
          </div>
          <Button
            variant="secondary"
            onClick={() => setApprove(true)}
            disabled={busy}
          >
            <Download size={16} />
            Install Required Components
          </Button>
        </Notice>
      ) : (
        <Notice tone="success">
          The connector and filesystem components are already installed.
        </Notice>
      )}
      {busy && <Spinner label="Checking your system…" />}
      <ConfirmationDialog
        open={approve}
        onOpenChange={setApprove}
        title="Install connection components?"
        description="These packages will be installed using your distribution’s package manager. Your desktop will ask you to authenticate."
      >
        <ul className="package-review">
          {inspection.packages.map((p) => (
            <li key={p}>
              <strong>{p}</strong> —{" "}
              {p === "rclone"
                ? "connects to your Google Drive"
                : "makes Drive available as a folder"}
            </li>
          ))}
        </ul>
        <div className="dialog-actions">
          <Button variant="secondary" onClick={() => setApprove(false)}>
            Cancel
          </Button>
          <Button
            onClick={() => {
              setApprove(false);
              install();
            }}
          >
            <ShieldCheck size={16} />
            Approve installation
          </Button>
        </div>
      </ConfirmationDialog>
    </>
  );
}
