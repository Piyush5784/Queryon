export type SslMode = "disable" | "prefer" | "require" | "verify-ca" | "verify-full";

export type Engine = "postgres" | "neon" | "my-sql";

export interface ConnectionProfile {
  id: string;
  name: string;
  engine: Engine;
  host: string;
  port: number;
  database: string;
  user: string;
  password: string;
  sslMode: SslMode;
}

export interface SavedConnectionProfile {
  id: string;
  name: string;
  // Optional to match the Rust side's #[serde(default)] — connections
  // saved before the engine field existed load without it.
  engine?: Engine;
  host: string;
  port: number;
  database: string;
  user: string;
  sslMode: SslMode;
}

export function engineOf(profile: { engine?: Engine }): Engine {
  return profile.engine ?? "postgres";
}

export const DEFAULT_PG_PORT = 5432;
export const DEFAULT_MYSQL_PORT = 3306;

export function defaultPortFor(engine: Engine): number {
  return engine === "my-sql" ? DEFAULT_MYSQL_PORT : DEFAULT_PG_PORT;
}

export function urlSchemeFor(engine: Engine): string {
  return engine === "my-sql" ? "mysql" : "postgres";
}

export function createEmptyConnectionDraft(engine: Engine): Omit<ConnectionProfile, "id"> {
  return {
    name: "",
    engine,
    host: "localhost",
    port: defaultPortFor(engine),
    database: "",
    user: "",
    password: "",
    // Neon requires TLS; everything else defaults to off for local/dev use.
    sslMode: engine === "neon" ? "require" : "disable",
  };
}

export function parseConnectionUrl(
  raw: string,
  engine: Engine
): Partial<Omit<ConnectionProfile, "id" | "engine">> | null {
  const trimmed = raw.trim();
  if (!trimmed) return null;

  let url: URL;
  try {
    url = new URL(trimmed);
  } catch {
    return null;
  }

  const validSchemes = engine === "my-sql" ? ["mysql:"] : ["postgres:", "postgresql:"];
  if (!validSchemes.includes(url.protocol)) {
    return null;
  }

  const database = decodeURIComponent(url.pathname.replace(/^\//, ""));
  const sslModeParam = url.searchParams.get("sslmode") ?? url.searchParams.get("ssl-mode");
  const sslMode = isSslMode(sslModeParam) ? sslModeParam : undefined;

  const result: Partial<Omit<ConnectionProfile, "id" | "engine">> = {
    host: url.hostname || undefined,
    port: url.port ? Number(url.port) : defaultPortFor(engine),
    database: database || undefined,
    user: url.username ? decodeURIComponent(url.username) : undefined,
    password: url.password ? decodeURIComponent(url.password) : undefined,
    sslMode,
  };

  return Object.fromEntries(
    Object.entries(result).filter(([, v]) => v !== undefined)
  ) as Partial<Omit<ConnectionProfile, "id" | "engine">>;
}

function isSslMode(value: string | null): value is SslMode {
  return (
    !!value &&
    ["disable", "prefer", "require", "verify-ca", "verify-full"].includes(value)
  );
}

export function toDisplayUrl(profile: {
  engine?: Engine;
  host: string;
  port: number;
  database: string;
  user: string;
  sslMode: SslMode;
}): string {
  const auth = profile.user ? `${profile.user}@` : "";
  const db = profile.database ? `/${profile.database}` : "";
  const ssl = profile.sslMode && profile.sslMode !== "disable" ? `?sslmode=${profile.sslMode}` : "";
  return `${urlSchemeFor(engineOf(profile))}://${auth}${profile.host}:${profile.port}${db}${ssl}`;
}
