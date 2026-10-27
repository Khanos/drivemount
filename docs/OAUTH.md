# Google OAuth and public distribution

No real credentials are included. Personal OAuth is the default supported setup path. The wizard links directly to Google Cloud pages in the system browser and explains Desktop app registration, consent, Drive API enablement, and test users.

## Runtime protocol

1. Create a unique connection draft and an empty private rclone configuration file.
2. Start an owned, short-lived `rclone rcd` with that configuration and a Unix socket under a mode-0700 directory. Its output is not logged.
3. Send `config/create` with the Drive backend, custom client fields, `scope=drive`, and `opt.nonInteractive=true`, `opt.obscure=true` over the socket.
4. Answer only recognized protocol questions through `config/update` with `opt.continue`, `opt.state`, and `opt.result`. Local browser authorization is selected; Shared Drive selection is declined. Unexpected protocol questions fail closed instead of being guessed.
5. Let rclone perform browser authorization and token handling. Google’s login is never embedded in Tauri.
6. Treat an empty protocol state as completion, terminate the authorization helper, and make a read-only `rclone about --json` request with bounded retries and timeouts.
7. Mark the saved draft authorized only after verification succeeds.

Cancellation and the three-minute outer timeout drop the helper with kill-on-drop. Partial owned configuration stays in its private draft folder for retry. Nothing in the default user rclone configuration is overwritten. Renewal uses the saved owned configuration through `config/update` with `config_refresh_token=true`. Externally created remotes are renewed in their original tool.

Rclone owns the browser callback and its loopback listener. DriveMount never parses or forwards access tokens, refresh tokens, callback codes, or OAuth state to React. Credentials enter the local Tauri IPC call and the private rclone JSON request; they are not subprocess arguments or persisted frontend preferences. Memory zeroization is best effort for the original credential request and optional client registration; JSON serialization and third-party libraries may temporarily allocate other copies.

The installed rclone version determines the OAuth implementation. Do not claim DriveMount independently implements PKCE or controls rclone’s loopback port selection. Verify the exact shipped rclone OAuth behavior against Google's current requirements during release qualification.

## Application-owned client registration

For the one-click path, a distributor can supply Google's downloaded **Desktop** registration JSON, with top-level `installed`, at `/usr/share/drivemount/oauth-client.json`, or select a file with `DRIVEMOUNT_OAUTH_CLIENT_FILE`. A web registration with top-level `web` is rejected. The feature is advertised only when the file parses and contains plausible client fields. Invalid or absent registration leaves the personal-credentials guide available.

Desktop applications are public clients and cannot keep a distributed secret confidential. Never embed a private server/web client secret. The `client_secret` in a Desktop registration is a public-client parameter, not protection against impersonation. Keep actual user refresh tokens private regardless of the client registration model. [Google Desktop OAuth](https://developers.google.com/identity/protocols/oauth2/native-app)

## Public release gates

- Establish the final application identity, publisher, support contact, verified domains, privacy policy, and accurate consent-screen branding.
- Enable the Drive API in a publisher-owned Cloud project and create the correct Desktop OAuth registration.
- Justify and declare the requested Drive scopes in Cloud Console and the application. Both `drive` and `drive.readonly` are restricted scopes; read-only access does not automatically avoid restricted-scope review. The narrower `drive.file` scope cannot expose an arbitrary pre-existing full Drive. [Drive scopes](https://developers.google.com/workspace/drive/api/guides/api-specific-auth)
- Complete applicable brand and restricted-scope verification before broadly requesting access. Evaluate the precise security-assessment requirement: Google specifically distinguishes restricted data handled by servers from local-only operation. Do not assume every local desktop app needs a server assessment, or that local-only automatically exempts all review. [Restricted-scope verification](https://developers.google.com/identity/protocols/oauth2/production-readiness/restricted-scope-verification)
- Move the project to the appropriate production publishing status and validate access for accounts outside the test-user list. External projects in Testing commonly issue refresh tokens that expire after seven days when non-basic scopes are requested. [Google token expiration](https://developers.google.com/identity/protocols/oauth2#expiration)
- Test successful sign-in, user denial, browser closure, cancellation, timeout, invalid client, callback-port conflicts, token expiry/revocation, and reauthentication against a disposable Google account.
- Test an actual mount, file-manager access, harmless disposable-file edits, pending-write safeguards, network outage/recovery, service restart, application closure, reopen, login startup, and reboot.
- Audit the exact rclone version’s OAuth redirect and PKCE behavior, available package versions, and release dependency policy. Do not rely on the retiring shared rclone client.

None of the real Google-account acceptance tests or publisher verification steps above were completed by the isolated test suite.
