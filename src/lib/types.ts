export type CacheSettings = {
  mode: "full" | "writes" | "minimal";
  sizeGib: number;
  ageHours: number;
};
export type Preferences = {
  autostart: boolean;
  readOnly: boolean;
  cache: CacheSettings;
};
export type Connection = {
  id: string;
  remote: string;
  configPath: string;
  mountPath: string;
  service: string;
  cachePath: string;
  socketPath: string;
  managed: boolean;
  ownedRemote: boolean;
  authorized: boolean;
  configured: boolean;
  preferences: Preferences;
};
export type Check = {
  id: string;
  label: string;
  state: "ready" | "install" | "attention" | "unsupported";
  detail: string;
};
export type Remote = { name: string; backend: string; configPath: string };
export type ExistingMount = {
  target: string;
  source: string;
  service: string | null;
};
export type Inspection = {
  distro: string;
  packageManager: string | null;
  packages: string[];
  checks: Check[];
  remotes: Remote[];
  mounts: ExistingMount[];
  services: {
    name: string;
    remote: string;
    mountPath: string;
    configPath: string | null;
    autostart: boolean;
    managed: boolean;
  }[];
  connections: Connection[];
  defaultMount: string;
  appOauthAvailable: boolean;
  freeCacheBytes: number;
};
export type ConnectionStatus = {
  id: string;
  state:
    | "connected"
    | "connecting"
    | "reconnecting"
    | "disconnected"
    | "offline"
    | "authenticationRequired"
    | "error";
  mounted: boolean;
  activeState: string;
  autostart: boolean;
  pendingUploads: number | null;
  openFiles: number | null;
  cacheBytes: number | null;
  detail: string;
};
export type AuthStatus = {
  id: string;
  state: "waiting" | "completing" | "connected" | "cancelled" | "failed";
  message: string;
};
export type AppError = { code: string; message: string; recoverable: boolean };
export type PathValidation = {
  path: string;
  valid: boolean;
  reason: string;
  exists: boolean;
};
export type Progress = { stage: string; message: string };
export type SetupRequest = {
  id: string;
  mountPath: string;
  preferences: Preferences;
};
export type AuthRequest = {
  clientId: string;
  clientSecret: string;
  connectionId: string | null;
};
export interface Commands {
  inspect_system: { args: undefined; result: Inspection };
  install_dependencies: { args: { consent: boolean }; result: void };
  begin_auth: { args: { request: AuthRequest }; result: AuthStatus };
  auth_status: { args: undefined; result: AuthStatus | null };
  cancel_auth: { args: undefined; result: void };
  validate_mount_path: { args: { path: string }; result: PathValidation };
  import_remote: {
    args: { name: string; consent: boolean };
    result: Connection;
  };
  connect_drive: { args: { request: SetupRequest }; result: ConnectionStatus };
  setup_progress: { args: undefined; result: Progress };
  connection_status: { args: { id: string }; result: ConnectionStatus };
  connection_action: {
    args: {
      id: string;
      action: "start" | "stop" | "restart";
      acknowledgeUnknown: boolean;
    };
    result: ConnectionStatus;
  };
  set_autostart: { args: { id: string; enabled: boolean }; result: void };
  read_diagnostics: {
    args: { id: string };
    result: { report: string; details: string };
  };
  open_drive: { args: { id: string }; result: void };
  choose_folder: { args: undefined; result: string | null };
  open_help: {
    args: { page: "cloud" | "api" | "consent" | "credentials" | "guide" };
    result: void;
  };
}
