import { useState } from "react";
import { CheckCircle2, Link2, Loader2, XCircle } from "lucide-react";

import { Button } from "@/src/app/components/ui/button";
import { Checkbox } from "@/src/app/components/ui/checkbox";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/src/app/components/ui/dialog";
import {
  Field,
  FieldContent,
  FieldDescription,
  FieldGroup,
  FieldLabel,
  FieldSeparator,
} from "@/src/app/components/ui/field";
import { Input } from "@/src/app/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/src/app/components/ui/select";
import { connect, saveConnection, testConnection } from "@/src/features/connections/api";
import {
  createEmptyConnectionDraft,
  parsePostgresUrl,
  type ConnectionProfile,
  type SslMode,
} from "@/src/features/connections/types";
import { toPrefixedErrorMessage } from "@/src/lib/tauri/errors";

interface ConnectionDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConnected: (profile: ConnectionProfile) => void;
}

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

const DEV_DB_URL = "postgres://devuser:devpass@localhost:55433/devdb";

export function ConnectionDialog({
  open,
  onOpenChange,
  onConnected,
}: ConnectionDialogProps) {
  const [url, setUrl] = useState(DEV_DB_URL);
  const [draft, setDraft] = useState(() => ({
    ...createEmptyConnectionDraft(),
    ...parsePostgresUrl(DEV_DB_URL),
  }));
  const [status, setStatus] = useState<Status>({ kind: "idle" });
  const [saveForNextTime, setSaveForNextTime] = useState(true);

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
    const parsed = parsePostgresUrl(value);
    if (parsed) {
      setDraft((prev) => ({ ...prev, ...parsed }));
    }
  }

  function reset() {
    setUrl(DEV_DB_URL);
    setDraft({ ...createEmptyConnectionDraft(), ...parsePostgresUrl(DEV_DB_URL) });
    setStatus({ kind: "idle" });
  }

  function handleOpenChange(next: boolean) {
    if (!next) reset();
    onOpenChange(next);
  }

  function buildProfile(): ConnectionProfile {
    return {
      id: crypto.randomUUID(),
      ...draft,
      name: draft.name.trim(),
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

  const canSave = draft.name.trim().length > 0 && draft.host.trim().length > 0;
  const busy = status.kind === "testing" || status.kind === "connecting";

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>New Connection</DialogTitle>
          <DialogDescription>
            Connect to a PostgreSQL database. Paste a connection URL to
            auto-fill the fields, or enter them manually.
          </DialogDescription>
        </DialogHeader>

        <FieldGroup>
          <Field>
            <FieldLabel htmlFor="conn-url">
              <Link2 className="size-3.5" />
              Connection URL
            </FieldLabel>
            <FieldContent>
              <Input
                id="conn-url"
                placeholder="postgres://user:password@host:5432/database"
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
                placeholder="postgres"
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
                  placeholder="postgres"
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
        </FieldGroup>

        {status.kind === "test-success" && (
          <div className="flex items-center gap-2 rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-3 py-2 text-sm text-emerald-600 dark:text-emerald-400">
            <CheckCircle2 className="size-4 shrink-0" />
            Connected — PostgreSQL {status.serverVersion}
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
      </DialogContent>
    </Dialog>
  );
}
