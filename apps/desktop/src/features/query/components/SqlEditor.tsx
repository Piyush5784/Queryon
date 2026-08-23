import { useMemo, useRef } from "react";
import CodeMirror, { type KeyBinding, keymap } from "@uiw/react-codemirror";
import { sql, PostgreSQL, type SQLNamespace } from "@codemirror/lang-sql";
import { EditorView } from "@codemirror/view";

interface SqlEditorProps {
  value: string;
  onChange: (value: string) => void;
  onExecute: () => void;
  disabled?: boolean;
  schema?: SQLNamespace;
}

export function SqlEditor({ value, onChange, onExecute, disabled, schema }: SqlEditorProps) {
  const onExecuteRef = useRef(onExecute);
  onExecuteRef.current = onExecute;

  const extensions = useMemo(() => {
    const executeBinding: KeyBinding[] = [
      {
        key: "Mod-Enter",
        run: () => {
          onExecuteRef.current();
          return true;
        },
      },
    ];
    return [
      sql({ dialect: PostgreSQL, schema, upperCaseKeywords: true }),
      keymap.of(executeBinding),
      EditorView.lineWrapping,
    ];
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [schema]);

  return (
    <CodeMirror
      value={value}
      onChange={onChange}
      theme="dark"
      editable={!disabled}
      height="100%"
      extensions={extensions}
      basicSetup={{
        lineNumbers: true,
        foldGutter: false,
        highlightActiveLine: true,
        autocompletion: true,
        closeBrackets: false,
      }}
      className="h-full text-sm [&_.cm-editor]:h-full [&_.cm-scroller]:font-mono"
    />
  );
}
