import { Database, Zap } from "lucide-react";

import { cn } from "@/src/lib/utils";
import type { Engine } from "@/src/features/connections/types";

export const ENGINE_OPTIONS: { value: Engine; label: string }[] = [
  { value: "postgres", label: "PostgreSQL" },
  { value: "my-sql", label: "MySQL" },
  { value: "sqlite", label: "SQLite" },
  { value: "sql-server", label: "SQL Server" },
  { value: "neon", label: "Neon" },
  { value: "cockroach-db", label: "CockroachDB" },
  { value: "greengage-db", label: "GreengageDB" },
  { value: "maria-db", label: "MariaDB" },
  { value: "ti-db", label: "TiDB" },
  { value: "star-rocks", label: "StarRocks" },
  { value: "click-house", label: "ClickHouse" },
  { value: "duck-db", label: "DuckDB" },
];

const ENGINE_COLORS: Record<Engine, string> = {
  postgres: "text-blue-500",
  "my-sql": "text-orange-500",
  sqlite: "text-slate-500",
  "sql-server": "text-red-600",
  neon: "text-emerald-500",
  "cockroach-db": "text-rose-500",
  "greengage-db": "text-teal-500",
  "maria-db": "text-sky-600",
  "ti-db": "text-violet-500",
  "star-rocks": "text-amber-600",
  "click-house": "text-yellow-500",
  "duck-db": "text-yellow-600",
};

interface EngineIconProps {
  engine: Engine;
  className?: string;
}

export function EngineIcon({ engine, className }: EngineIconProps) {
  const Icon = engine === "neon" ? Zap : Database;
  return <Icon className={cn(ENGINE_COLORS[engine], className)} />;
}
