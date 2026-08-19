export type SslMode = "disable" | "prefer" | "require" | "verify-ca" | "verify-full";

export interface ConnectionProfile {
  id: string;
  name: string;
  driver: "postgres";
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
  host: string;
  port: number;
  database: string;
  user: string;
  sslMode: SslMode;
}

export const DEFAULT_PG_PORT = 5432;

export function createEmptyConnectionDraft(): Omit<ConnectionProfile, "id"> {
  return {
    name: "",
    driver: "postgres",
    host: "localhost",
    port: DEFAULT_PG_PORT,
    database: "",
    user: "",
    password: "",
    sslMode: "disable",
  };
}

export function parsePostgresUrl(
  raw: string
): Partial<Omit<ConnectionProfile, "id" | "driver">> | null {
  const trimmed = raw.trim();
  if (!trimmed) return null;

  let url: URL;
  try {
    url = new URL(trimmed);
  } catch {
    return null;
  }

  if (!["postgres:", "postgresql:"].includes(url.protocol)) {
    return null;
  }

  const database = decodeURIComponent(url.pathname.replace(/^\//, ""));
  const sslModeParam = url.searchParams.get("sslmode");
  const sslMode = isSslMode(sslModeParam) ? sslModeParam : undefined;

  const result: Partial<Omit<ConnectionProfile, "id" | "driver">> = {
    host: url.hostname || undefined,
    port: url.port ? Number(url.port) : DEFAULT_PG_PORT,
    database: database || undefined,
    user: url.username ? decodeURIComponent(url.username) : undefined,
    password: url.password ? decodeURIComponent(url.password) : undefined,
    sslMode,
  };

  return Object.fromEntries(
    Object.entries(result).filter(([, v]) => v !== undefined)
  ) as Partial<Omit<ConnectionProfile, "id" | "driver">>;
}

function isSslMode(value: string | null): value is SslMode {
  return (
    !!value &&
    ["disable", "prefer", "require", "verify-ca", "verify-full"].includes(value)
  );
}

export function toDisplayUrl(profile: {
  host: string;
  port: number;
  database: string;
  user: string;
  sslMode: SslMode;
}): string {
  const auth = profile.user ? `${profile.user}@` : "";
  const db = profile.database ? `/${profile.database}` : "";
  const ssl = profile.sslMode && profile.sslMode !== "disable" ? `?sslmode=${profile.sslMode}` : "";
  return `postgres://${auth}${profile.host}:${profile.port}${db}${ssl}`;
}
