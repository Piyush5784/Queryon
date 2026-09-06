import { useEffect, useSyncExternalStore } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

import { toErrorMessage } from "@/src/lib/tauri/errors";

export type UpdaterStatus =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "up-to-date" }
  | { kind: "updating"; progress: number }
  | { kind: "ready" }
  | { kind: "error"; message: string };

let status: UpdaterStatus = { kind: "idle" };
const listeners = new Set<() => void>();
let idleResetTimer: ReturnType<typeof setTimeout> | null = null;

function setStatus(next: UpdaterStatus) {
  status = next;
  listeners.forEach((listener) => listener());
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

function getSnapshot() {
  return status;
}

export function useUpdaterStatus() {
  return useSyncExternalStore(subscribe, getSnapshot);
}

function clearIdleReset() {
  if (idleResetTimer) {
    clearTimeout(idleResetTimer);
    idleResetTimer = null;
  }
}

async function installUpdate(update: Update) {
  setStatus({ kind: "updating", progress: 0 });
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
        setStatus({ kind: "updating", progress });
        break;
      }
      case "Finished":
        break;
    }
  });
  setStatus({ kind: "ready" });
}

async function runCheck(manual: boolean) {
  if (status.kind === "checking" || status.kind === "updating") return;
  clearIdleReset();
  setStatus({ kind: "checking" });
  try {
    const update = await check();
    if (!update) {
      if (manual) {
        setStatus({ kind: "up-to-date" });
        idleResetTimer = setTimeout(() => setStatus({ kind: "idle" }), 2500);
      } else {
        setStatus({ kind: "idle" });
      }
      return;
    }
    await installUpdate(update);
  } catch (err) {
    setStatus({ kind: "error", message: toErrorMessage(err) });
  }
}

export function checkForUpdates() {
  runCheck(true);
}

export async function restartToUpdate() {
  await relaunch();
}

export function UpdateChecker() {
  useEffect(() => {
    runCheck(false);
  }, []);

  return null;
}
