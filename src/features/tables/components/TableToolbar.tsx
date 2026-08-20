import { ArrowUpDown, Columns3, ListFilter } from "lucide-react";

import { Button } from "@/src/app/components/ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "@/src/app/components/ui/popover";
import { TableColumnsPanel } from "@/src/features/tables/components/TableColumnsPanel";
import type { TableFilter, TableSort } from "@/src/features/tables/api";

interface TableToolbarProps {
  columns: string[];
  filters: TableFilter[];
  showFilters: boolean;
  onToggleFilters: () => void;
  sort: TableSort[];
  showSort: boolean;
  onToggleSort: () => void;
  hiddenColumns: Set<string>;
  onHiddenColumnsChange: (hiddenColumns: Set<string>) => void;
}

export function TableToolbar({
  columns,
  filters,
  showFilters,
  onToggleFilters,
  sort,
  showSort,
  onToggleSort,
  hiddenColumns,
  onHiddenColumnsChange,
}: TableToolbarProps) {
  return (
    <div className="flex shrink-0 items-center gap-1.5 border-b px-3 py-1.5">
      <Button
        variant={showFilters || filters.length > 0 ? "secondary" : "outline"}
        size="xs"
        className="gap-1.5"
        onClick={onToggleFilters}
      >
        <ListFilter className="size-3.5" />
        Filters
        {filters.length > 0 && (
          <span className="rounded-full bg-primary px-1.5 text-[0.65rem] text-primary-foreground">
            {filters.length}
          </span>
        )}
      </Button>

      <Button
        variant={showSort || sort.length > 0 ? "secondary" : "outline"}
        size="xs"
        className="gap-1.5"
        onClick={onToggleSort}
      >
        <ArrowUpDown className="size-3.5" />
        Sort
        {sort.length > 0 && (
          <span className="rounded-full bg-primary px-1.5 text-[0.65rem] text-primary-foreground">
            {sort.length}
          </span>
        )}
      </Button>

      <Popover>
        <PopoverTrigger
          render={
            <Button variant={hiddenColumns.size > 0 ? "secondary" : "outline"} size="xs" className="gap-1.5">
              <Columns3 className="size-3.5" />
              Columns
              {hiddenColumns.size > 0 && (
                <span className="rounded-full bg-primary px-1.5 text-[0.65rem] text-primary-foreground">
                  {columns.length - hiddenColumns.size}
                </span>
              )}
            </Button>
          }
        />
        <PopoverContent align="start" className="w-auto">
          <TableColumnsPanel columns={columns} hiddenColumns={hiddenColumns} onChange={onHiddenColumnsChange} />
        </PopoverContent>
      </Popover>
    </div>
  );
}
