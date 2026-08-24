import { Button } from "@queryon/ui/components/button";
import { Checkbox } from "@queryon/ui/components/checkbox";

interface TableColumnsPanelProps {
  columns: string[];
  hiddenColumns: Set<string>;
  onChange: (hiddenColumns: Set<string>) => void;
}

export function TableColumnsPanel({ columns, hiddenColumns, onChange }: TableColumnsPanelProps) {
  function toggle(column: string, checked: boolean) {
    const next = new Set(hiddenColumns);
    if (checked) next.delete(column);
    else next.add(column);
    onChange(next);
  }

  return (
    <div className="flex w-[220px] flex-col gap-1">
      <div className="max-h-64 overflow-y-auto">
        {columns.map((col) => (
          <label
            key={col}
            className="flex cursor-pointer items-center gap-2 rounded-md px-1.5 py-1 text-xs hover:bg-muted"
          >
            <Checkbox
              checked={!hiddenColumns.has(col)}
              onCheckedChange={(checked) => toggle(col, checked === true)}
            />
            <span className="truncate">{col}</span>
          </label>
        ))}
      </div>
      <Button
        variant="ghost"
        size="xs"
        className="self-start"
        onClick={() => onChange(new Set())}
        disabled={hiddenColumns.size === 0}
      >
        Show all
      </Button>
    </div>
  );
}
