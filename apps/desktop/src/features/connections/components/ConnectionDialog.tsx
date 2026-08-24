import { useState } from "react";
import { ArrowLeft, CheckCircle2, FolderOpen, Link2, Loader2, XCircle } from "lucide-react";

import { Button } from "@queryon/ui/components/button";
import { Checkbox } from "@queryon/ui/components/checkbox";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@queryon/ui/components/dialog";
import {
  Field,
  FieldContent,
  FieldDescription,
  FieldGroup,
  FieldLabel,
  FieldSeparator,
} from "@queryon/ui/components/field";
import { Input } from "@queryon/ui/components/input";
import { RadioGroup, RadioGroupItem } from "@queryon/ui/components/radio-group";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@queryon/ui/components/select";
import { connect, pickDuckdbFile, pickSshKeyFile, pickSqliteFile, saveConnection, testConnection } from "@/src/features/connections/api";
import {
  createEmptyConnectionDraft,
  isFileBasedEngine,
  parseConnectionUrl,
  urlSchemeFor,
  type ConnectionProfile,
  type Engine,
  type SshAuth,
  type SslMode,
} from "@/src/features/connections/types";
import { EngineIcon, ENGINE_OPTIONS } from "@/src/features/connections/components/EngineIcon";
import { toPrefixedErrorMessage } from "@/src/lib/tauri/errors";

interface ConnectionDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConnected: (profile: ConnectionProfile) => void;
}

type Screen = "type" | "details";

type Status =
  | { kind: "idle" }
  | { kind: "testing" | "connecting" }
  | { kind: "test-success"; serverVersion: string }
  | { kind: "error"; message: string };

const SSL_MODES: { value: SslMode; label: string }[] = [
  { value: "disable", label: "Disable" },
  { value: "prefer", label: "Prefer" },
  { value: "require", label: "Require" },
  { value: "verify-ca", label: "Verify-ca" },
  { value: "verify-full", label: "Verify-full" },
];

const DEV_URLS: Record<Engine, string> = {
  postgres: "postgres://devuser:devpass@localhost:55434/devdb",
  "my-sql": "mysql://devuser:devpass@localhost:33066/devdb",
  sqlite: "",
  neon: "postgres://devuser:devpass@localhost:55434/devdb",
  "cockroach-db": "postgres://devuser@localhost:26257/devdb?sslmode=disable",
  "greengage-db": "postgres://devuser:devpass@localhost:55433/devdb",
  "maria-db": "mysql://devuser:devpass@localhost:33077/devdb",
  "ti-db": "mysql://devuser:devpass@localhost:44000/devdb",
  "sql-server": "",
  "star-rocks": "mysql://root:devpass@localhost:39030/devdb",
  "click-house": "http://devuser:devpass@localhost:48123/devdb",
  "duck-db": "",
};

const FIELD_PLACEHOLDERS: Record<Engine, { database: string; user: string }> = {
  postgres: { database: "postgres", user: "postgres" },
  neon: { database: "postgres", user: "postgres" },
  "cockroach-db": { database: "postgres", user: "postgres" },
  "greengage-db": { database: "postgres", user: "postgres" },
  "my-sql": { database: "mysql", user: "root" },
  "maria-db": { database: "mysql", user: "root" },
  "ti-db": { database: "mysql", user: "root" },
  "star-rocks": { database: "mysql", user: "root" },
  sqlite: { database: "postgres", user: "postgres" },
  "sql-server": { database: "master", user: "sa" },
  "click-house": { database: "default", user: "default" },
  "duck-db": { database: "postgres", user: "postgres" },
};

export function ConnectionDialog({
  open,
  onOpenChange,
  onConnected,
}: ConnectionDialogProps) {
  const [screen, setScreen] = useState<Screen>("type");
  const [engine, setEngine] = useState<Engine>("postgres");
  const [url, setUrl] = useState("");
  const [draft, setDraft] = useState(() => createEmptyConnectionDraft("postgres"));
  const [status, setStatus] = useState<Status>({ kind: "idle" });
  const [saveForNextTime, setSaveForNextTime] = useState(true);
  const [useSshTunnel, setUseSshTunnel] = useState(false);
  const [sshHost, setSshHost] = useState("");
  const [sshPort, setSshPort] = useState(22);
  const [sshUsername, setSshUsername] = useState("");
  const [sshAuthKind, setSshAuthKind] = useState<SshAuth["kind"]>("privateKey");
  const [sshPassword, setSshPassword] = useState("");
  const [sshKeyPath, setSshKeyPath] = useState("");
  const [sshPassphrase, setSshPassphrase] = useState("");

  function update<K extends keyof typeof draft>(
    key: K,
    value: (typeof draft)[K],
  ) {
    setDraft((prev) => ({ ...prev, [key]: value }));
    setStatus({ kind: "idle" });
  }

  function handleUrlChange(value: string) {
    setUrl(value);
    setStatus({ kind: "idle" });
    const parsed = parseConnectionUrl(value, engine);
    if (parsed) {
      setDraft((prev) => ({ ...prev, ...parsed }));
    }
  }

  function handleSelectEngine(next: Engine) {
    setEngine(next);
    const prefill = DEV_URLS[next];
    setUrl(prefill);
    setDraft({ ...createEmptyConnectionDraft(next), ...parseConnectionUrl(prefill, next) });
    setStatus({ kind: "idle" });
    setScreen("details");
  }

  function reset() {
    setScreen("type");
    setUrl("");
    setStatus({ kind: "idle" });
    setUseSshTunnel(false);
    setSshHost("");
    setSshPort(22);
    setSshUsername("");
    setSshAuthKind("privateKey");
    setSshPassword("");
    setSshKeyPath("");
    setSshPassphrase("");
  }

  function handleOpenChange(next: boolean) {
    if (!next) reset();
    onOpenChange(next);
  }

  async function handleChooseKeyFile() {
    try {
      const chosen = await pickSshKeyFile();
      if (chosen) setSshKeyPath(chosen);
    } catch (err) {
      setStatus({ kind: "error", message: toPrefixedErrorMessage("Failed to open file picker", err) });
    }
  }

  async function handleChooseDatabaseFile() {
    try {
      const chosen = engine === "duck-db" ? await pickDuckdbFile() : await pickSqliteFile();
      if (chosen) {
        update("database", chosen);
        if (!draft.name.trim()) {
          const fileName = chosen.split(/[/\\]/).pop() ?? chosen;
          update("name", fileName.replace(/\.(db|sqlite|sqlite3|duckdb)$/i, ""));
        }
      }
    } catch (err) {
      setStatus({ kind: "error", message: toPrefixedErrorMessage("Failed to open file picker", err) });
    }
  }

  function buildProfile(): ConnectionProfile {
    const auth: SshAuth =
      sshAuthKind === "password"
        ? { kind: "password", password: sshPassword }
        : { kind: "privateKey", keyPath: sshKeyPath, passphrase: sshPassphrase };

    return {
      id: crypto.randomUUID(),
      ...draft,
      name: draft.name.trim(),
      sshTunnel: useSshTunnel
        ? { host: sshHost.trim(), port: sshPort, username: sshUsername.trim(), auth }
        : null,
    };
  }

  async function handleTest() {
    setStatus({ kind: "testing" });
    try {
      const info = await testConnection(buildProfile());
      setStatus({ kind: "test-success", serverVersion: info.serverVersion });
    } catch (err) {
      setStatus({
        kind: "error",
        message: toPrefixedErrorMessage("Failed to test connection", err),
      });
    }
  }

  async function handleConnect() {
    if (!canSave) return;
    setStatus({ kind: "connecting" });
    const profile = buildProfile();
    try {
      await connect(profile);
      if (saveForNextTime) {
        await saveConnection(profile);
      }
      onConnected(profile);
      handleOpenChange(false);
    } catch (err) {
      setStatus({
        kind: "error",
        message: toPrefixedErrorMessage("Failed to connect", err),
      });
    }
  }

  const sshTunnelValid =
    !useSshTunnel ||
    (sshHost.trim().length > 0 &&
      sshUsername.trim().length > 0 &&
      (sshAuthKind === "password" ? sshPassword.length > 0 : sshKeyPath.trim().length > 0));
  const requiredFieldFilled = isFileBasedEngine(engine)
    ? draft.database.trim().length > 0
    : draft.host.trim().length > 0;
  const canSave = draft.name.trim().length > 0 && requiredFieldFilled && sshTunnelValid;
  const busy = status.kind === "testing" || status.kind === "connecting";
  const engineLabel = ENGINE_OPTIONS.find((o) => o.value === engine)?.label ?? "database";

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent
        className={screen === "type" || !useSshTunnel ? "sm:max-w-md" : "sm:max-w-3xl"}
      >
        {screen === "type" ? (
          <>
            <DialogHeader>
              <DialogTitle>New Connection</DialogTitle>
              <DialogDescription>Choose the type of database you want to connect to.</DialogDescription>
            </DialogHeader>

            <div className="grid grid-cols-3 gap-3">
              {ENGINE_OPTIONS.map((option) => (
                <button
                  key={option.value}
                  type="button"
                  onClick={() => handleSelectEngine(option.value)}
                  className="flex flex-col items-center gap-2 rounded-lg border border-border p-4 text-center transition-colors hover:border-primary/50 hover:bg-accent"
                >
                  <EngineIcon engine={option.value} className="size-8" />
                  <span className="text-sm font-medium">{option.label}</span>
                </button>
              ))}
            </div>
          </>
        ) : (
          <>
            <DialogHeader>
              <div className="flex items-center gap-2">
                <Button
                  variant="ghost"
                  size="icon-xs"
                  className="shrink-0"
                  onClick={() => setScreen("type")}
                >
                  <ArrowLeft className="size-3.5" />
                </Button>
                <DialogTitle>New {engineLabel} Connection</DialogTitle>
              </div>
              <DialogDescription>
                {isFileBasedEngine(engine)
                  ? `Choose a ${engineLabel} database file on this machine.`
                  : engine === "neon"
                    ? "Paste your Neon connection string to auto-fill the fields, or enter them manually."
                    : `Connect to a ${engineLabel} database. Paste a connection URL to auto-fill the fields, or enter them manually.`}
              </DialogDescription>
            </DialogHeader>

            <div className={useSshTunnel ? "grid grid-cols-2 gap-6" : "grid grid-cols-1"}>
              <FieldGroup>
                {isFileBasedEngine(engine) ? (
                  <>
                    <Field>
                      <FieldLabel htmlFor="conn-file-path">Database File</FieldLabel>
                      <FieldContent>
                        <div className="flex gap-2">
                          <Input
                            id="conn-file-path"
                            placeholder="/path/to/database.db"
                            value={draft.database}
                            onChange={(e) => update("database", e.target.value)}
                            autoComplete="off"
                            spellCheck={false}
                            className="flex-1"
                          />
                          <Button
                            type="button"
                            variant="outline"
                            size="sm"
                            className="gap-1.5"
                            onClick={handleChooseDatabaseFile}
                          >
                            <FolderOpen className="size-3.5" />
                            Choose
                          </Button>
                        </div>
                      </FieldContent>
                    </Field>

                    <Field>
                      <FieldLabel htmlFor="conn-name">Name</FieldLabel>
                      <FieldContent>
                        <Input
                          id="conn-name"
                          placeholder="My Database"
                          value={draft.name}
                          onChange={(e) => update("name", e.target.value)}
                          autoComplete="off"
                        />
                      </FieldContent>
                    </Field>
                  </>
                ) : (
                  <>
                    <Field>
                      <FieldLabel htmlFor="conn-url">
                        <Link2 className="size-3.5" />
                        Connection URL
                      </FieldLabel>
                      <FieldContent>
                        <Input
                          id="conn-url"
                          placeholder={`${urlSchemeFor(engine)}://user:password@host:${draft.port}/database`}
                          value={url}
                          onChange={(e) => handleUrlChange(e.target.value)}
                          autoComplete="off"
                          spellCheck={false}
                        />
                        <FieldDescription>
                          Optional — fills in the fields below automatically.
                        </FieldDescription>
                      </FieldContent>
                    </Field>

                    <FieldSeparator>or enter manually</FieldSeparator>

                    <Field>
                      <FieldLabel htmlFor="conn-name">Name</FieldLabel>
                      <FieldContent>
                        <Input
                          id="conn-name"
                          placeholder="My Database"
                          value={draft.name}
                          onChange={(e) => update("name", e.target.value)}
                          autoComplete="off"
                        />
                      </FieldContent>
                    </Field>

                    <div className="grid grid-cols-3 gap-3">
                      <Field className="col-span-2">
                        <FieldLabel htmlFor="conn-host">Host</FieldLabel>
                        <FieldContent>
                          <Input
                            id="conn-host"
                            placeholder="localhost"
                            value={draft.host}
                            onChange={(e) => update("host", e.target.value)}
                            autoComplete="off"
                            spellCheck={false}
                          />
                        </FieldContent>
                      </Field>
                      <Field>
                        <FieldLabel htmlFor="conn-port">Port</FieldLabel>
                        <FieldContent>
                          <Input
                            id="conn-port"
                            type="number"
                            value={draft.port}
                            onChange={(e) => update("port", Number(e.target.value) || 0)}
                          />
                        </FieldContent>
                      </Field>
                    </div>

                    <Field>
                      <FieldLabel htmlFor="conn-database">Database</FieldLabel>
                      <FieldContent>
                        <Input
                          id="conn-database"
                          placeholder={FIELD_PLACEHOLDERS[engine].database}
                          value={draft.database}
                          onChange={(e) => update("database", e.target.value)}
                          autoComplete="off"
                          spellCheck={false}
                        />
                      </FieldContent>
                    </Field>

                    <div className="grid grid-cols-2 gap-3">
                      <Field>
                        <FieldLabel htmlFor="conn-user">User</FieldLabel>
                        <FieldContent>
                          <Input
                            id="conn-user"
                            placeholder={FIELD_PLACEHOLDERS[engine].user}
                            value={draft.user}
                            onChange={(e) => update("user", e.target.value)}
                            autoComplete="off"
                            spellCheck={false}
                          />
                        </FieldContent>
                      </Field>
                      <Field>
                        <FieldLabel htmlFor="conn-password">Password</FieldLabel>
                        <FieldContent>
                          <Input
                            id="conn-password"
                            type="password"
                            value={draft.password}
                            onChange={(e) => update("password", e.target.value)}
                            autoComplete="off"
                          />
                        </FieldContent>
                      </Field>
                    </div>

                    <Field>
                      <FieldLabel htmlFor="conn-ssl">SSL Mode</FieldLabel>
                      <FieldContent>
                        <Select
                          value={draft.sslMode}
                          onValueChange={(value) => update("sslMode", value as SslMode)}
                        >
                          <SelectTrigger id="conn-ssl">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            {SSL_MODES.map((mode) => (
                              <SelectItem key={mode.value} value={mode.value}>
                                {mode.label}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      </FieldContent>
                    </Field>
                  </>
                )}
                <Field orientation="horizontal">
                  <Checkbox
                    id="conn-save"
                    checked={saveForNextTime}
                    onCheckedChange={(checked) => setSaveForNextTime(checked === true)}
                  />
                  <FieldLabel htmlFor="conn-save" className="font-normal">
                    Save this connection for next time
                  </FieldLabel>
                </Field>
                <Field orientation="horizontal">
                  <Checkbox
                    id="conn-read-only"
                    checked={draft.readOnly}
                    onCheckedChange={(checked) => update("readOnly", checked === true)}
                  />
                  <FieldContent>
                    <FieldLabel htmlFor="conn-read-only" className="font-normal">
                      Read-only
                    </FieldLabel>
                    <FieldDescription>
                      Blocks every write — editing rows, running DDL, executing non-SELECT SQL — on this connection.
                    </FieldDescription>
                  </FieldContent>
                </Field>
                {!isFileBasedEngine(engine) && (
                  <Field orientation="horizontal">
                    <Checkbox
                      id="conn-ssh-tunnel"
                      checked={useSshTunnel}
                      onCheckedChange={(checked) => setUseSshTunnel(checked === true)}
                    />
                    <FieldContent>
                      <FieldLabel htmlFor="conn-ssh-tunnel" className="font-normal">
                        Use SSH Tunnel
                      </FieldLabel>
                      <FieldDescription>
                        Route this connection through a bastion host — needed when the database isn't directly reachable.
                      </FieldDescription>
                    </FieldContent>
                  </Field>
                )}
              </FieldGroup>

              {useSshTunnel && !isFileBasedEngine(engine) && (
                <FieldGroup className="border-l pl-6">
                  <div className="grid grid-cols-3 gap-3">
                    <Field className="col-span-2">
                      <FieldLabel htmlFor="ssh-host">SSH Host</FieldLabel>
                      <FieldContent>
                        <Input
                          id="ssh-host"
                          placeholder="bastion.example.com"
                          value={sshHost}
                          onChange={(e) => setSshHost(e.target.value)}
                          autoComplete="off"
                          spellCheck={false}
                        />
                      </FieldContent>
                    </Field>
                    <Field>
                      <FieldLabel htmlFor="ssh-port">Port</FieldLabel>
                      <FieldContent>
                        <Input
                          id="ssh-port"
                          type="number"
                          value={sshPort}
                          onChange={(e) => setSshPort(Number(e.target.value) || 0)}
                        />
                      </FieldContent>
                    </Field>
                  </div>

                  <Field>
                    <FieldLabel htmlFor="ssh-username">SSH User</FieldLabel>
                    <FieldContent>
                      <Input
                        id="ssh-username"
                        placeholder="ec2-user"
                        value={sshUsername}
                        onChange={(e) => setSshUsername(e.target.value)}
                        autoComplete="off"
                        spellCheck={false}
                      />
                    </FieldContent>
                  </Field>

                  <Field>
                    <FieldLabel>Auth</FieldLabel>
                    <FieldContent>
                      <RadioGroup
                        value={sshAuthKind}
                        onValueChange={(v) => setSshAuthKind(v as SshAuth["kind"])}
                      >
                        <Field orientation="horizontal">
                          <RadioGroupItem value="privateKey" id="ssh-auth-key" />
                          <FieldLabel htmlFor="ssh-auth-key" className="font-normal">
                            Private Key
                          </FieldLabel>
                        </Field>
                        <Field orientation="horizontal">
                          <RadioGroupItem value="password" id="ssh-auth-password" />
                          <FieldLabel htmlFor="ssh-auth-password" className="font-normal">
                            Password
                          </FieldLabel>
                        </Field>
                      </RadioGroup>
                    </FieldContent>
                  </Field>

                  {sshAuthKind === "privateKey" ? (
                    <>
                      <Field>
                        <FieldLabel htmlFor="ssh-key-path">Key File</FieldLabel>
                        <FieldContent>
                          <div className="flex gap-2">
                            <Input
                              id="ssh-key-path"
                              placeholder="~/.ssh/id_ed25519"
                              value={sshKeyPath}
                              onChange={(e) => setSshKeyPath(e.target.value)}
                              autoComplete="off"
                              spellCheck={false}
                              className="flex-1"
                            />
                            <Button
                              type="button"
                              variant="outline"
                              size="sm"
                              className="gap-1.5"
                              onClick={handleChooseKeyFile}
                            >
                              <FolderOpen className="size-3.5" />
                              Choose
                            </Button>
                          </div>
                        </FieldContent>
                      </Field>
                      <Field>
                        <FieldLabel htmlFor="ssh-passphrase">Passphrase</FieldLabel>
                        <FieldContent>
                          <Input
                            id="ssh-passphrase"
                            type="password"
                            placeholder="Optional"
                            value={sshPassphrase}
                            onChange={(e) => setSshPassphrase(e.target.value)}
                            autoComplete="off"
                          />
                        </FieldContent>
                      </Field>
                    </>
                  ) : (
                    <Field>
                      <FieldLabel htmlFor="ssh-password">SSH Password</FieldLabel>
                      <FieldContent>
                        <Input
                          id="ssh-password"
                          type="password"
                          value={sshPassword}
                          onChange={(e) => setSshPassword(e.target.value)}
                          autoComplete="off"
                        />
                      </FieldContent>
                    </Field>
                  )}
                </FieldGroup>
              )}
            </div>

            {status.kind === "test-success" && (
              <div className="flex items-center gap-2 rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-3 py-2 text-sm text-emerald-600 dark:text-emerald-400">
                <CheckCircle2 className="size-4 shrink-0" />
                Connected — {status.serverVersion}
              </div>
            )}
            {status.kind === "error" && (
              <div className="flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
                <XCircle className="size-4 shrink-0 translate-y-0.5" />
                <span className="wrap-break-word">{status.message}</span>
              </div>
            )}

            <DialogFooter>
              <Button
                variant="outline"
                onClick={() => handleOpenChange(false)}
                disabled={busy}
              >
                Cancel
              </Button>
              <Button
                variant="outline"
                onClick={handleTest}
                disabled={!canSave || busy}
              >
                {status.kind === "testing" && (
                  <Loader2 className="size-4 animate-spin" />
                )}
                Test Connection
              </Button>
              <Button onClick={handleConnect} disabled={!canSave || busy}>
                {status.kind === "connecting" && (
                  <Loader2 className="size-4 animate-spin" />
                )}
                Connect
              </Button>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}
