import { CheckCircle2, Folder, FolderOpen, Home } from "lucide-react";
import { useEffect, useState } from "react";
import { api, asError } from "../lib/api";
import type { AppError, PathValidation } from "../lib/types";
import { Button } from "./ui/button";
import { Notice, Spinner } from "./common";
export function FolderPicker({
  value,
  onChange,
  onValidity,
  onError,
}: {
  value: string;
  onChange: (v: string) => void;
  onValidity: (valid: boolean) => void;
  onError: (e: AppError) => void;
}) {
  const [result, setResult] = useState<PathValidation | null>(null);
  const [busy, setBusy] = useState(true);
  useEffect(() => {
    let live = true;
    setBusy(true);
    onValidity(false);
    const timer = setTimeout(() => {
      void api("validate_mount_path", { path: value })
        .then((v) => {
          if (live) {
            setResult(v);
            onValidity(v.valid);
          }
        })
        .catch((e) => {
          if (live) {
            onValidity(false);
            onError(asError(e));
          }
        })
        .finally(() => {
          if (live) setBusy(false);
        });
    }, 350);
    return () => {
      live = false;
      clearTimeout(timer);
    };
  }, [value, onValidity, onError]);
  return (
    <>
      <div className="location-preview">
        <div className="folder-art">
          <Folder size={72} strokeWidth={1.2} />
          <span>
            <CheckCircle2 size={22} />
          </span>
        </div>
        <div className="breadcrumb">
          <Home size={15} />
          {value.startsWith("/home/") ? "Home" : "Computer"}
          <span>›</span>
          <strong>
            {value
              .split("/")
              .filter(Boolean)
              .slice(value.startsWith("/home/") ? 2 : 0)
              .join(" / ") || "GoogleDrive"}
          </strong>
        </div>
      </div>
      <label htmlFor="mount-path">Drive folder</label>
      <div className="folder-input">
        <input
          id="mount-path"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          spellCheck={false}
        />
        <Button
          variant="secondary"
          onClick={() =>
            void api("choose_folder")
              .then((p) => {
                if (p) onChange(p);
              })
              .catch((e) => onError(asError(e)))
          }
        >
          <FolderOpen size={17} />
          Browse
        </Button>
      </div>
      <div className="validation" aria-live="polite">
        {busy ? (
          <Spinner label="Checking this folder…" />
        ) : (
          <span className={result?.valid ? "valid" : "invalid"}>
            {result?.valid && <CheckCircle2 size={15} />} {result?.reason}
          </span>
        )}
      </div>
      <Notice>
        Choose an empty folder. Your existing local files will stay safe, and
        your Drive files will load from the cloud as you open them.
      </Notice>
    </>
  );
}
