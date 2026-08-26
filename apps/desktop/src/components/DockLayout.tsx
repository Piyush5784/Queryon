import { forwardRef, useEffect, useImperativeHandle, useRef, useState } from "react";
import { DockviewReact, type DockviewApi, type DockviewReadyEvent, type IDockviewPanelProps } from "dockview-react";

import { cn } from "@queryon/ui/lib/utils";

import "dockview-react/dist/styles/dockview.css";
import "@/src/components/dock-theme.css";

export type DockLayoutPreset = "stacked" | "side-by-side" | "tabbed";

interface DockPanelDef<T> {
  id: string;
  title: string;
  params: T;
}

interface DockLayoutProps<T> {
  storageKey: string;
  panels: DockPanelDef<T>[];
  render: (params: T) => React.ReactNode;
  className?: string;
}

export interface DockLayoutHandle {
  applyLayout: (preset: DockLayoutPreset) => void;
}

function DockPanelContent({ params }: IDockviewPanelProps<{ render: () => React.ReactNode }>) {
  return <div className="h-full min-h-0 overflow-hidden">{params.render()}</div>;
}

function DockLayoutInner<T>(
  { storageKey, panels, render, className }: DockLayoutProps<T>,
  ref: React.ForwardedRef<DockLayoutHandle>
) {
  const apiRef = useRef<DockviewApi | null>(null);
  const panelsRef = useRef(panels);
  panelsRef.current = panels;
  const renderRef = useRef(render);
  renderRef.current = render;
  const [ready, setReady] = useState(false);

  function buildLayout(api: DockviewApi, preset: DockLayoutPreset) {
    const direction = preset === "side-by-side" ? "right" : preset === "tabbed" ? "within" : "below";
    panelsRef.current.forEach((panel, index) => {
      api.addPanel({
        id: panel.id,
        title: panel.title,
        component: "content",
        params: { render: () => renderRef.current(panelsRef.current[index]!.params) },
        position: index === 0 ? undefined : { referencePanel: panelsRef.current[0]!.id, direction },
      });
    });
  }

  function onReady(event: DockviewReadyEvent) {
    apiRef.current = event.api;

    const savedLayout = localStorage.getItem(storageKey);
    if (savedLayout) {
      try {
        event.api.fromJSON(JSON.parse(savedLayout));
        setReady(true);
        return;
      } catch {
        localStorage.removeItem(storageKey);
      }
    }

    buildLayout(event.api, "stacked");
    setReady(true);
  }

  useImperativeHandle(ref, () => ({
    applyLayout(preset: DockLayoutPreset) {
      const api = apiRef.current;
      if (!api) return;
      localStorage.removeItem(storageKey);
      api.clear();
      buildLayout(api, preset);
    },
  }));

  useEffect(() => {
    if (!apiRef.current) return;
    for (const panel of panels) {
      const dockPanel = apiRef.current.getPanel(panel.id);
      dockPanel?.api.updateParameters({ render: () => render(panel.params) });
    }
  });

  useEffect(() => {
    const api = apiRef.current;
    if (!api || !ready) return;
    const disposable = api.onDidLayoutChange(() => {
      localStorage.setItem(storageKey, JSON.stringify(api.toJSON()));
    });
    return () => disposable.dispose();
  }, [ready, storageKey]);

  return (
    <DockviewReact
      className={cn("dockview-theme-dark dockview-theme-queryon overflow-hidden", className)}
      components={{ content: DockPanelContent }}
      onReady={onReady}
    />
  );
}

export const DockLayout = forwardRef(DockLayoutInner) as <T>(
  props: DockLayoutProps<T> & { ref?: React.ForwardedRef<DockLayoutHandle> }
) => ReturnType<typeof DockLayoutInner>;
