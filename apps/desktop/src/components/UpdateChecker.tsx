import { useEffect, useState } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogMedia,
  AlertDialogTitle,
} from "@queryon/ui/components/alert-dialog";
import { DownloadCloud } from "lucide-react";
import { toErrorMessage } from "@/src/lib/tauri/errors";

type Status =
  | { kind: "idle" }
  | { kind: "available"; update: Update }
  | { kind: "installing"; progress: number }
  | { kind: "error"; message: string };

export function UpdateChecker() {
  const [status, setStatus] = useState<Status>({ kind: "idle" });

  useEffect(() => {
    let cancelled = false;

    check()
      .then((update) => {
        if (!cancelled && update) {
          setStatus({ kind: "available", update });
        }
      })
      .catch(() => {
        // No update endpoint configured yet, or the check failed —
        // silent, never interrupts startup for this.
      });

    return () => {
      cancelled = true;
    };
  }, []);

  if (status.kind === "idle") {
    return null;
  }

  async function handleInstall() {
    if (status.kind !== "available") return;
    const update = status.update;
    setStatus({ kind: "installing", progress: 0 });

    try {
      let downloaded = 0;
      let contentLength = 0;
      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            contentLength = event.data.contentLength ?? 0;
            break;
          case "Progress": {
            downloaded += event.data.chunkLength;
            const progress = contentLength > 0 ? Math.round((downloaded / contentLength) * 100) : 0;
            setStatus({ kind: "installing", progress });
            break;
          }
          case "Finished":
            break;
        }
      });
      await relaunch();
    } catch (err) {
      setStatus({ kind: "error", message: toErrorMessage(err) });
    }
  }

  return (
    <AlertDialog open onOpenChange={(open) => !open && setStatus({ kind: "idle" })}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogMedia className="bg-primary/10 text-primary">
            <DownloadCloud />
          </AlertDialogMedia>
          <AlertDialogTitle>
            {status.kind === "available" && `Update available: v${status.update.version}`}
            {status.kind === "installing" && "Installing update…"}
            {status.kind === "error" && "Update failed"}
          </AlertDialogTitle>
          <AlertDialogDescription>
            {status.kind === "available" &&
              (status.update.body || "A new version of Queryon is ready to install.")}
            {status.kind === "installing" && `Downloading and installing (${status.progress}%). The app will restart when finished.`}
            {status.kind === "error" && status.message}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          {status.kind === "available" && (
            <>
              <AlertDialogCancel>Later</AlertDialogCancel>
              <AlertDialogAction onClick={handleInstall}>Install and restart</AlertDialogAction>
            </>
          )}
          {status.kind === "error" && (
            <AlertDialogAction onClick={() => setStatus({ kind: "idle" })}>Close</AlertDialogAction>
          )}
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
