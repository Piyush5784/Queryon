export function toErrorMessage(err: unknown): string {
  if (typeof err === "string" && err.trim()) {
    return err;
  }

  if (err instanceof Error) {
    if (/reading 'invoke'|invoke is not a function/i.test(err.message)) {
      return "Not running inside the desktop app shell — launch with `tauri dev` (not a browser) to connect to a database.";
    }
    return err.message;
  }

  if (err && typeof err === "object" && "message" in err) {
    const msg = (err as { message?: unknown }).message;
    if (typeof msg === "string" && msg.trim()) return msg;
  }

  return "Something went wrong. Please try again.";
}

export function toPrefixedErrorMessage(prefix: string, err: unknown): string {
  return `${prefix}: ${toErrorMessage(err)}`;
}
