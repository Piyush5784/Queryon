import dagre from "@dagrejs/dagre";
import type { Edge, Node } from "@xyflow/react";

import type { ConstraintInfo } from "@/src/features/schema/api";
import type { ColumnInfo } from "@/src/features/tables/api";

export interface TableColumnData {
  name: string;
  dataType: string;
  isNullable: boolean;
  isPrimaryKey: boolean;
  isForeignKey: boolean;
}

export interface TableNodeData extends Record<string, unknown> {
  table: string;
  columns: TableColumnData[];
}

const NODE_WIDTH = 240;
const ROW_HEIGHT = 24;
const HEADER_HEIGHT = 32;

export function buildSchemaGraph(
  tables: { name: string; columns: ColumnInfo[]; constraints: ConstraintInfo[] }[]
): { nodes: Node<TableNodeData>[]; edges: Edge[] } {
  const foreignKeyColumns = new Map<string, Set<string>>();
  for (const { name, constraints } of tables) {
    const fkCols = new Set<string>();
    for (const constraint of constraints) {
      if (constraint.kind === "foreign-key") {
        for (const col of constraint.columns) fkCols.add(col);
      }
    }
    foreignKeyColumns.set(name, fkCols);
  }

  const nodes: Node<TableNodeData>[] = tables.map(({ name, columns }) => {
    const fkCols = foreignKeyColumns.get(name) ?? new Set();
    return {
      id: name,
      type: "table",
      position: { x: 0, y: 0 },
      data: {
        table: name,
        columns: columns.map((c) => ({
          name: c.name,
          dataType: c.dataType,
          isNullable: c.isNullable,
          isPrimaryKey: c.isPrimaryKey,
          isForeignKey: fkCols.has(c.name),
        })),
      },
    };
  });

  const edges: Edge[] = [];
  for (const { name, constraints } of tables) {
    for (const constraint of constraints) {
      if (constraint.kind !== "foreign-key" || !constraint.referencedTable) continue;
      const sourceColumn = constraint.columns[0];
      const targetColumn = constraint.referencedColumns[0];
      edges.push({
        id: `${name}.${constraint.name}`,
        source: name,
        target: constraint.referencedTable,
        sourceHandle: sourceColumn ? `${name}.${sourceColumn}.source` : undefined,
        targetHandle: targetColumn ? `${constraint.referencedTable}.${targetColumn}.target` : undefined,
        label: constraint.columns.join(", "),
        type: "step",
        animated: false,
      });
    }
  }

  return { nodes: layoutNodes(nodes, edges), edges };
}

function layoutNodes(nodes: Node<TableNodeData>[], edges: Edge[]): Node<TableNodeData>[] {
  const graph = new dagre.graphlib.Graph();
  graph.setDefaultEdgeLabel(() => ({}));
  graph.setGraph({ rankdir: "LR", nodesep: 60, ranksep: 120 });

  for (const node of nodes) {
    const height = HEADER_HEIGHT + node.data.columns.length * ROW_HEIGHT + 8;
    graph.setNode(node.id, { width: NODE_WIDTH, height });
  }
  for (const edge of edges) {
    graph.setEdge(edge.source, edge.target);
  }

  dagre.layout(graph);

  return nodes.map((node) => {
    const pos = graph.node(node.id);
    return {
      ...node,
      position: { x: pos.x - NODE_WIDTH / 2, y: pos.y - pos.height / 2 },
    };
  });
}
