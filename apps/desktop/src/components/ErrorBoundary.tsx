import { Component, type ErrorInfo, type ReactNode } from "react";
import { AlertTriangle } from "lucide-react";

import { Button } from "@queryon/ui/components/button";
import { logFrontendError } from "@/src/lib/tauri/commands";

interface ErrorBoundaryProps {
  children: ReactNode;
}

interface ErrorBoundaryState {
  error: Error | null;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { error: null };

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    logFrontendError(error.message, `${error.stack ?? ""}\n${info.componentStack ?? ""}`).catch(() => {});
  }

  render() {
    if (!this.state.error) {
      return this.props.children;
    }

    return (
      <div className="flex h-svh flex-col items-center justify-center gap-4 bg-background p-6 text-center">
        <AlertTriangle className="size-10 text-destructive" />
        <div className="flex flex-col gap-1">
          <p className="text-lg font-medium">Something went wrong</p>
          <p className="max-w-md text-sm text-muted-foreground">
            Queryon hit an unexpected error and can't continue. This has been written to the log
            file so it can be looked into.
          </p>
        </div>
        <Button onClick={() => window.location.reload()}>Reload</Button>
      </div>
    );
  }
}
