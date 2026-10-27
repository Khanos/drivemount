import {
  ArrowUpRight,
  Check,
  KeyRound,
  LockKeyhole,
  Mail,
  ShieldCheck,
} from "lucide-react";
import { useEffect, useState } from "react";
import { api, asError } from "../lib/api";
import { credentialError } from "../lib/store";
import type { AppError, AuthStatus } from "../lib/types";
import { Button } from "./ui/button";
import { ExternalHelp, Notice, Spinner } from "./common";
export function OAuthConnectionCard({
  appOAuth,
  connectionId,
  onConnected,
  onError,
  reauth = false,
}: {
  reauth?: boolean;
  appOAuth: boolean;
  connectionId: string | null;
  onConnected: (id: string) => void;
  onError: (e: AppError) => void;
}) {
  const [clientId, setClientId] = useState("");
  const [secret, setSecret] = useState("");
  const [personal, setPersonal] = useState(!appOAuth && !reauth);
  const [auth, setAuth] = useState<AuthStatus | null>(null);
  const [busy, setBusy] = useState(false);
  const waiting = auth?.state === "waiting" || auth?.state === "completing";
  useEffect(() => {
    if (!waiting) return;
    let live = true;
    const tick = async () => {
      try {
        const s = await api("auth_status");
        if (!live || !s) return;
        setAuth(s);
        if (s.state === "connected") onConnected(s.id);
      } catch (e) {
        if (live) onError(asError(e));
      }
    };
    const timer = setInterval(() => void tick(), 1000);
    return () => {
      live = false;
      clearInterval(timer);
    };
  }, [waiting, onConnected, onError]);
  async function start() {
    if (personal && !reauth) {
      const error = credentialError(clientId, secret);
      if (error) {
        onError({
          code: "oauth_credentials",
          message: error,
          recoverable: true,
        });
        return;
      }
    }
    setBusy(true);
    try {
      setAuth(
        await api("begin_auth", {
          request: {
            clientId: personal ? clientId.trim() : "",
            clientSecret: personal ? secret : "",
            connectionId: connectionId ?? auth?.id ?? null,
          },
        }),
      );
      setSecret("");
      setClientId("");
    } catch (e) {
      onError(asError(e));
    } finally {
      setBusy(false);
    }
  }
  const help = (page: "cloud" | "api" | "consent" | "credentials" | "guide") =>
    void api("open_help", { page }).catch((e) => onError(asError(e)));
  return (
    <>
      <div className="oauth-card">
        <div className="account-emblem">
          <Mail size={28} />
        </div>
        <h2>Your files. Your account.</h2>
        <p>
          Sign in securely in your browser. Your Google password stays with
          Google.
        </p>
        <div className="permission-note">
          <ShieldCheck size={16} />
          Drive access, with your permission
        </div>
      </div>
      {personal && (
        <div className="personal-oauth">
          <div className="inline-heading">
            <KeyRound size={18} />
            <h3>A one-time Google setup</h3>
          </div>
          <p>
            This independent build needs a personal Google OAuth client. You
            create it once, then connect here.
          </p>
          <details className="oauth-guide" open>
            <summary>Walk me through it</summary>
            <ol>
              <li>
                <ExternalHelp onClick={() => help("cloud")}>
                  Create or select a Google Cloud project
                </ExternalHelp>
              </li>
              <li>
                <ExternalHelp onClick={() => help("api")}>
                  Enable the Google Drive API
                </ExternalHelp>
              </li>
              <li>
                <ExternalHelp onClick={() => help("consent")}>
                  Configure Google Auth Platform
                </ExternalHelp>
                <p>
                  Choose External, add your account as a test user if in
                  Testing, and configure the Google Drive permission. Testing
                  authorizations may expire after seven days.
                </p>
              </li>
              <li>
                <ExternalHelp onClick={() => help("credentials")}>
                  Create an OAuth client
                </ExternalHelp>
                <p>
                  Choose <strong>Desktop app</strong>. Copy its client ID and
                  client secret below.
                </p>
              </li>
            </ol>
            <ExternalHelp onClick={() => help("guide")}>
              Open the full rclone guide
            </ExternalHelp>
          </details>
          <div className="credential-fields">
            <label htmlFor="client-id">
              Client ID
              <input
                id="client-id"
                value={clientId}
                onChange={(e) => setClientId(e.target.value)}
                autoComplete="off"
                spellCheck={false}
                placeholder="…apps.googleusercontent.com"
                disabled={waiting}
              />
            </label>
            <label htmlFor="client-secret">
              Client secret
              <input
                id="client-secret"
                type="password"
                value={secret}
                onChange={(e) => setSecret(e.target.value)}
                autoComplete="off"
                placeholder="Your Desktop app client secret"
                disabled={waiting}
              />
            </label>
          </div>
          <p className="help">
            <LockKeyhole size={13} />
            Credentials stay on your computer. They are never included in
            diagnostic reports.
          </p>
        </div>
      )}
      {waiting ? (
        <Notice>
          <Spinner label={auth?.message} />
          <Button
            variant="ghost"
            size="small"
            onClick={() =>
              void api("cancel_auth")
                .then(() =>
                  setAuth({
                    ...auth!,
                    state: "cancelled",
                    message:
                      "Google account connection was cancelled. You can try again.",
                  }),
                )
                .catch((e) => onError(asError(e)))
            }
          >
            Cancel authorization
          </Button>
        </Notice>
      ) : auth?.state === "connected" ? (
        <Notice tone="success">
          <Check size={16} />
          Your Google account is connected.
        </Notice>
      ) : (
        <>
          <Button className="wide" onClick={() => void start()} disabled={busy}>
            {busy ? (
              <Spinner label="Opening secure sign-in…" />
            ) : (
              <>
                <Mail size={18} />
                Connect with Google
                <ArrowUpRight size={15} />
              </>
            )}
          </Button>
          {auth && (
            <p className="help" role="status">
              {auth.message}
            </p>
          )}
        </>
      )}
      {appOAuth && !reauth && (
        <Button
          variant="ghost"
          size="small"
          disabled={waiting}
          onClick={() => setPersonal(!personal)}
        >
          {personal
            ? "Use the application’s Google connection"
            : "Use my own OAuth credentials"}
        </Button>
      )}
    </>
  );
}
