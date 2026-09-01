import { useMemo } from "react";
import CodeMirror from "@uiw/react-codemirror";
import { sql, PostgreSQL, type SQLNamespace } from "@codemirror/lang-sql";
import { EditorView } from "@codemirror/view";

interface SqlEditorProps {
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
  schema?: SQLNamespace;
}

export function SqlEditor({ value, onChange, disabled, schema }: SqlEditorProps) {
  const extensions = useMemo(() => {
    return [
      sql({ dialect: PostgreSQL, schema, upperCaseKeywords: true }),
      EditorView.lineWrapping,
    ];
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
