import type { DdlStatement, NewColumn, NewConstraint, NewIndex } from "@/src/features/schema/api";

export interface StagedNewColumn extends NewColumn {
  tempId: string;
}

export interface StagedColumnEdit {
  currentName: string;
  column: NewColumn;
}

export interface StagedNewIndex extends NewIndex {
  tempId: string;
}

export interface StagedNewConstraint extends NewConstraint {
  tempId: string;
}

export interface SchemaChanges {
  newColumns: StagedNewColumn[];
  droppedColumns: string[];
  editedColumns: StagedColumnEdit[];
  newIndexes: StagedNewIndex[];
  droppedIndexes: string[];
  newConstraints: StagedNewConstraint[];
  droppedConstraints: string[];
}

export function emptyChanges(): SchemaChanges {
  return {
    newColumns: [],
    droppedColumns: [],
    editedColumns: [],
    newIndexes: [],
    droppedIndexes: [],
    newConstraints: [],
    droppedConstraints: [],
  };
}

export function hasChanges(changes: SchemaChanges): boolean {
  return (
    changes.newColumns.length > 0 ||
    changes.droppedColumns.length > 0 ||
    changes.editedColumns.length > 0 ||
    changes.newIndexes.length > 0 ||
    changes.droppedIndexes.length > 0 ||
    changes.newConstraints.length > 0 ||
    changes.droppedConstraints.length > 0
  );
}

export function changeCount(changes: SchemaChanges): number {
  return (
    changes.newColumns.length +
    changes.droppedColumns.length +
    changes.editedColumns.length +
    changes.newIndexes.length +
    changes.droppedIndexes.length +
    changes.newConstraints.length +
    changes.droppedConstraints.length
  );
}

export function isColumnValid(column: StagedNewColumn): boolean {
  return column.name.trim().length > 0 && column.dataType.trim().length > 0;
}

export function isColumnEditValid(edit: StagedColumnEdit): boolean {
  return edit.column.name.trim().length > 0 && edit.column.dataType.trim().length > 0;
}

export function isIndexValid(index: StagedNewIndex): boolean {
  return index.name.trim().length > 0 && index.columns.length > 0;
}

export function isConstraintValid(constraint: StagedNewConstraint): boolean {
  if (!constraint.name.trim()) return false;
  if (constraint.kind === "check") return !!constraint.checkExpression?.trim();
  if (constraint.kind === "foreign-key") {
    return (
      constraint.columns.length > 0 &&
      !!constraint.referencedTable &&
      constraint.referencedColumns.length > 0
    );
  }
  return constraint.columns.length > 0;
}

export function allValid(changes: SchemaChanges): boolean {
  return (
    changes.newColumns.every(isColumnValid) &&
    changes.editedColumns.every(isColumnEditValid) &&
    changes.newIndexes.every(isIndexValid) &&
    changes.newConstraints.every(isConstraintValid)
  );
}

export function toStatements(table: string, changes: SchemaChanges): DdlStatement[] {
  const statements: DdlStatement[] = [];

  for (const column of changes.droppedColumns) {
    statements.push({ op: "dropColumn", table, column });
  }
  for (const index of changes.droppedIndexes) {
    statements.push({ op: "dropIndex", table, index });
  }
  for (const constraint of changes.droppedConstraints) {
    statements.push({ op: "dropConstraint", table, constraint });
  }
  for (const edit of changes.editedColumns) {
    statements.push({ op: "alterColumn", table, edit });
  }
  for (const { tempId: _tempId, ...column } of changes.newColumns) {
    statements.push({ op: "addColumn", table, column });
  }
  for (const { tempId: _tempId, ...index } of changes.newIndexes) {
    statements.push({ op: "addIndex", table, index });
  }
  for (const { tempId: _tempId, ...constraint } of changes.newConstraints) {
    statements.push({ op: "addConstraint", table, constraint });
  }

  return statements;
}

let nextTempId = 0;
export function makeTempId(): string {
  nextTempId += 1;
  return `tmp_${nextTempId}`;
}
