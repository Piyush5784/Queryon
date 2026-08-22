import { useState } from "react";
import { Download } from "lucide-react";

import { Button } from "@/src/app/components/ui/button";
import { ExportDialog } from "@/src/components/ExportDialog";
import type { ExportTarget } from "@/src/components/exportTypes";

interface ExportButtonProps {
  target: ExportTarget;
  fileBaseName: string;
  disabled?: boolean;
}

export function ExportButton({ target, fileBaseName, disabled }: ExportButtonProps) {
  const [open, setOpen] = useState(false);

  const isDisabled =
    disabled ||
    (target.kind === "rows" && target.rows.length === 0) ||
    (target.kind === "query" && target.page.rows.length === 0);

  return (
    <>
      <Button variant="outline" size="xs" className="gap-1.5" disabled={isDisabled} onClick={() => setOpen(true)}>
        <Download className="size-3.5" />
        Download
      </Button>
      <ExportDialog open={open} onOpenChange={setOpen} target={target} fileBaseName={fileBaseName} />
    </>
  );
}
