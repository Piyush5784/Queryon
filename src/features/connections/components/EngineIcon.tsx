import { Database, Zap } from "lucide-react";

import { cn } from "@/src/lib/utils";
import type { Engine } from "@/src/features/connections/types";

export const ENGINE_OPTIONS: { value: Engine; label: string }[] = [
  { value: "postgres", label: "PostgreSQL" },
  { value: "my-sql", label: "MySQL" },
  { value: "neon", label: "Neon" },
  { value: "cockroach-db", label: "CockroachDB" },
  { value: "maria-db", label: "MariaDB" },
];

const ENGINE_COLORS: Record<Engine, string> = {
  postgres: "text-blue-500",
  "my-sql": "text-orange-500",
  neon: "text-emerald-500",
  "cockroach-db": "text-rose-500",
  "maria-db": "text-sky-600",
};

interface EngineIconProps {
  engine: Engine;
  className?: string;
}

export function EngineIcon({ engine, className }: EngineIconProps) {
  const Icon = engine === "neon" ? Zap : Database;
  return <Icon className={cn(ENGINE_COLORS[engine], className)} />;
}
