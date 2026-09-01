import ReactDOM from "react-dom/client";
import App from "@/src/app/App";
import { DetachedTabApp } from "@/src/app/DetachedTabApp";
import { readDetachedTabFromUrl } from "@/src/app/detachedWindow";
import { ErrorBoundary } from "@/src/components/ErrorBoundary";
import { logFrontendError } from "@/src/lib/tauri/commands";

window.addEventListener("error", (event) => {
  logFrontendError(event.message, event.error?.stack).catch(() => {});
});

window.addEventListener("unhandledrejection", (event) => {
  const reason = event.reason;
  const message = reason instanceof Error ? reason.message : String(reason);
  const stack = reason instanceof Error ? reason.stack : undefined;
  logFrontendError(`unhandled rejection: ${message}`, stack).catch(() => {});
});

const detachedTab = readDetachedTabFromUrl();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <ErrorBoundary>{detachedTab ? <DetachedTabApp tab={detachedTab} /> : <App />}</ErrorBoundary>
);
