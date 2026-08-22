import type { Engine } from "@/src/features/connections/types";

export interface SchemaCapabilities {
  primaryKeyHasCustomName: boolean;
}

const POSTGRES_CAPABILITIES: SchemaCapabilities = {
  primaryKeyHasCustomName: true,
};

const MYSQL_CAPABILITIES: SchemaCapabilities = {
  primaryKeyHasCustomName: false,
};

export function capabilitiesFor(engine: Engine): SchemaCapabilities {
  return engine === "my-sql" || engine === "maria-db" ? MYSQL_CAPABILITIES : POSTGRES_CAPABILITIES;
}
