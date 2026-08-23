import { useCallback, useEffect, useMemo, useRef, useState, type RefObject } from "react";
import {
  applyNodeChanges,
  Background,
  Controls,
  ControlButton,
  MiniMap,
  ReactFlow,
  type Edge,
  type Node,
  type NodeChange,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { KeyRound, Link2, Loader2, Maximize, Minimize } from "lucide-react";
import { listConstraints, type ConstraintInfo } from "@/src/features/schema/api";
import { buildSchemaGraph, type TableNodeData } from "@/src/features/schema/graph";
import { TableNode } from "@/src/features/schema/components/TableNode";
import { getTableColumns, listTables, type ColumnInfo } from "@/src/features/tables/api";
import { toErrorMessage } from "@/src/lib/tauri/errors";

function useResolvedColorMode(): "dark" | "light" {
  const [mode, setMode] = useState<"dark" | "light">(() =>
    document.documentElement.classList.contains("dark") ? "dark" : "light"
  );

  useEffect(() => {
    const root = document.documentElement;
    const observer = new MutationObserver(() => {
      setMode(root.classList.contains("dark") ? "dark" : "light");
    });
    observer.observe(root, { attributes: true, attributeFilter: ["class"] });
    return () => observer.disconnect();
  }, []);

  return mode;
}

function FullscreenControlButton({ targetRef }: { targetRef: RefObject<HTMLDivElement | null> }) {
  const [isFullscreen, setIsFullscreen] = useState(false);

  useEffect(() => {
    function handleChange() {
      setIsFullscreen(!!document.fullscreenElement);
    }
    document.addEventListener("fullscreenchange", handleChange);
    return () => document.removeEventListener("fullscreenchange", handleChange);
  }, []);

  function toggle() {
    if (document.fullscreenElement) {
      document.exitFullscreen();
    } else {
      targetRef.current?.requestFullscreen();
    }
  }

  return (
    <ControlButton title={isFullscreen ? "Exit fullscreen" : "Fullscreen"} onClick={toggle}>
      {isFullscreen ? <Minimize /> : <Maximize />}
    </ControlButton>
  );
}

interface SchemaGraphViewProps {
  connectionId: string;
  schema: string;
}

const NODE_TYPES = { table: TableNode };

export function SchemaGraphView({ connectionId, schema }: SchemaGraphViewProps) {
  const wrapperRef = useRef<HTMLDivElement>(null);
  const colorMode = useResolvedColorMode();
  const [nodes, setNodes] = useState<Node<TableNodeData>[] | null>(null);
  const [edges, setEdges] = useState<Edge[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    setNodes(null);
    setEdges(null);

    listTables(connectionId)
      .then(async (allTables) => {
        const tablesInSchema = allTables.filter((t) => t.schema === schema && t.kind === "table");
        const withColumns = await Promise.all(
          tablesInSchema.map(async (t) => {
            const [columns, constraints]: [ColumnInfo[], ConstraintInfo[]] = await Promise.all([
              getTableColumns(connectionId, schema, t.name),
              listConstraints(connectionId, schema, t.name),
            ]);
            return { name: t.name, columns, constraints };
          })
        );
        if (cancelled) return;
        const graph = buildSchemaGraph(withColumns);
        setNodes(graph.nodes);
        setEdges(graph.edges);
      })
      .catch((err) => {
        if (!cancelled) setError(toErrorMessage(err));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [connectionId, schema]);

  const nodeTypes = useMemo(() => NODE_TYPES, []);

  const onNodesChange = useCallback((changes: NodeChange<Node<TableNodeData>>[]) => {
    setNodes((prev) => (prev ? applyNodeChanges(changes, prev) : prev));
  }, []);

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center text-muted-foreground">
        <Loader2 className="size-5 animate-spin" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-full items-center justify-center p-6 text-center text-sm text-destructive">
        {error}
      </div>
    );
  }

  if (!nodes || nodes.length === 0) {
    return (
      <div className="flex h-full items-center justify-center p-6 text-center text-sm text-muted-foreground">
        No tables found in {schema}.
      </div>
    );
  }

  return (
    <div ref={wrapperRef} className="relative h-full bg-background">
      <ReactFlow
        nodes={nodes}
        edges={edges ?? []}
        nodeTypes={nodeTypes}
        onNodesChange={onNodesChange}
        nodesDraggable
        fitView
        minZoom={0.1}
        colorMode={colorMode}
        proOptions={{ hideAttribution: true }}
      >
        <Background gap={16} />
        <Controls showInteractive={false}>
          <FullscreenControlButton targetRef={wrapperRef} />
        </Controls>
        <MiniMap pannable zoomable className="bg-card!" />
      </ReactFlow>

      <div className="absolute top-3 left-3 z-10 flex items-center gap-3 rounded-lg border bg-card/95 px-2.5 py-1.5 text-xs shadow-sm backdrop-blur">
        <span className="flex items-center gap-1">
          <KeyRound className="size-3 text-amber-500" />
          Primary key
        </span>
        <span className="flex items-center gap-1">
          <Link2 className="size-3 text-primary" />
          Foreign key
        </span>
      </div>
    </div>
  );
}
