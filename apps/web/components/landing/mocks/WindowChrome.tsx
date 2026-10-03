export function WindowChrome({
  title,
  children,
  status,
}: {
  title: string;
  children: React.ReactNode;
  status?: string;
}) {
  return (
    <div className="overflow-hidden rounded-2xl border bg-card shadow-[0_30px_80px_-40px_rgba(5,46,22,0.45)] ring-1 ring-foreground/5">
      <div className="flex items-center gap-1.5 border-b bg-muted/50 px-4 py-2.5">
        <span className="size-2.5 rounded-full bg-[#ff5f57]" />
        <span className="size-2.5 rounded-full bg-[#febc2e]" />
        <span className="size-2.5 rounded-full bg-[#28c840]" />
        <span className="ml-3 font-mono text-xs text-muted-foreground">
          {title}
        </span>
        {status && (
          <span className="ml-auto flex items-center gap-1.5 rounded-full bg-brand-soft px-2 py-0.5 font-mono text-[10px] text-brand">
            <span className="size-1.5 rounded-full bg-brand" />
            {status}
          </span>
        )}
      </div>
      {children}
    </div>
  );
}
