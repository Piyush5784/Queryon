import { useRef, useState } from "react";
import { Copy } from "lucide-react";

import { Button } from "@queryon/ui/components/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@queryon/ui/components/dropdown-menu";

interface CopyButtonProps {
  columns: string[];
  rows: unknown[][];
  selectedRowIndices: Set<number>;
  disabled?: boolean;
}

function rowsToJson(columns: string[], rows: unknown[][]): string {
  const objects = rows.map((row) => {
    const obj: Record<string, unknown> = {};
    columns.forEach((col, i) => {
      obj[col] = row[i] ?? null;
    });
    return obj;
  });
  return JSON.stringify(objects, null, 2);
}

export function CopyButton({ columns, rows, selectedRowIndices, disabled }: CopyButtonProps) {
  const [copied, setCopied] = useState(false);
  const [open, setOpen] = useState(false);
  const closeTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  function cancelClose() {
    if (closeTimeoutRef.current) {
      clearTimeout(closeTimeoutRef.current);
      closeTimeoutRef.current = null;
    }
  }

  function openOnHover() {
    cancelClose();
    setOpen(true);
  }

  function closeOnHoverOut() {
    cancelClose();
    closeTimeoutRef.current = setTimeout(() => setOpen(false), 150);
  }

  async function copy(text: string) {
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  }

  function handleCopySelected() {
    const selectedRows = [...selectedRowIndices]
      .sort((a, b) => a - b)
      .map((i) => rows[i])
      .filter((row): row is unknown[] => row !== undefined);
    copy(rowsToJson(columns, selectedRows));
  }

  function handleCopyPage() {
    copy(rowsToJson(columns, rows));
  }

  return (
    <div>
      <DropdownMenu open={open} onOpenChange={setOpen}>
        <DropdownMenuTrigger
          render={
            <Button variant="outline" size="xs" onMouseEnter={openOnHover} onMouseLeave={closeOnHoverOut} className="gap-1.5 hover:border-none" disabled={disabled}>
              <Copy className="size-3.5" />
              {copied ? "Copied" : "Copy"}
            </Button>
          }
        />
        <DropdownMenuContent align="end" side="top">
          <DropdownMenuItem disabled={selectedRowIndices.size === 0} onClick={handleCopySelected}>
            Copy Selected Rows as JSON
          </DropdownMenuItem>
          <DropdownMenuItem onClick={handleCopyPage}>
            Copy Page ({rows.length} rows) as JSON
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}
