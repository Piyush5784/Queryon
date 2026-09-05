import { useEffect, useRef, useState } from "react";
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
  | { kind: "checking" }
  | { kind: "up-to-date" }
  | { kind: "available"; update: Update }
  | { kind: "installing"; progress: number }
  | { kind: "error"; message: string };

let triggerCheck: (() => void) | null = null;

export function checkForUpdates() {
  triggerCheck?.();
}

export function UpdateChecker() {
  const [status, setStatus] = useState<Status>({ kind: "idle" });
  const manualRef = useRef(false);

  async function runCheck(manual: boolean) {
    manualRef.current = manual;
    if (manual) setStatus({ kind: "checking" });
    try {
      const update = await check();
      if (update) {
        setStatus({ kind: "available", update });
      } else if (manual) {
        setStatus({ kind: "up-to-date" });
      } else {
        setStatus({ kind: "idle" });
      }
    } catch (err) {
      if (manual) setStatus({ kind: "error", message: toErrorMessage(err) });
    }
  }

  useEffect(() => {
    triggerCheck = () => runCheck(true);
    runCheck(false);
    return () => {
      triggerCheck = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
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
            {status.kind === "checking" && "Checking for updates…"}
            {status.kind === "up-to-date" && "You're up to date"}
            {status.kind === "available" && `Update available: v${status.update.version}`}
            {status.kind === "installing" && "Installing update…"}
            {status.kind === "error" && "Update failed"}
          </AlertDialogTitle>
          <AlertDialogDescription>
            {status.kind === "checking" && "Looking for a newer version of Queryon."}
            {status.kind === "up-to-date" && "You're already running the latest version of Queryon."}
            {status.kind === "available" &&
              (status.update.body || "A new version of Queryon is ready to install.")}
            {status.kind === "installing" && `Downloading and installing (${status.progress}%). The app will restart when finished.`}
            {status.kind === "error" && status.message}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          {(status.kind === "checking" || status.kind === "up-to-date" || status.kind === "error") && (
            <AlertDialogAction onClick={() => setStatus({ kind: "idle" })}>Close</AlertDialogAction>
          )}
          {status.kind === "available" && (
            <>
              <AlertDialogCancel>Later</AlertDialogCancel>
              <AlertDialogAction onClick={handleInstall}>Install and restart</AlertDialogAction>
            </>
          )}
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
