import type { TableTab } from "@/src/features/tables/types";
import type { QueryTab } from "@/src/features/query/types";
import type { CollectionTab } from "@/src/features/documents/types";

export type AppTab = TableTab | QueryTab | CollectionTab;
