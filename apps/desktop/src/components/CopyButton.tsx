import { SplitCopyButton } from "@/src/components/SplitCopyButton";

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
  const hasSelection = selectedRowIndices.size > 0;

  async function copySelected() {
    const selectedRows = [...selectedRowIndices]
      .sort((a, b) => a - b)
      .map((i) => rows[i])
      .filter((row): row is unknown[] => row !== undefined);
    await navigator.clipboard.writeText(rowsToJson(columns, selectedRows));
  }

  async function copyPage() {
    await navigator.clipboard.writeText(rowsToJson(columns, rows));
  }

  return (
    <SplitCopyButton
      disabled={disabled}
      onDefaultClick={hasSelection ? copySelected : copyPage}
      options={[
        { label: "Copy Selected Rows as JSON", onClick: copySelected, disabled: !hasSelection },
        { label: `Copy Page (${rows.length} rows) as JSON`, onClick: copyPage },
      ]}
    />
  );
}
