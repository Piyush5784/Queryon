export function WindowChrome({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="overflow-hidden rounded-xl border bg-card shadow-2xl shadow-foreground/5 ring-1 ring-foreground/5">
      <div className="flex items-center gap-1.5 border-b bg-muted/40 px-4 py-2.5">
        <span className="size-2.5 rounded-full bg-[#ff5f57]" />
        <span className="size-2.5 rounded-full bg-[#febc2e]" />
        <span className="size-2.5 rounded-full bg-[#28c840]" />
        <span className="ml-3 font-mono text-xs text-muted-foreground">{title}</span>
      </div>
      {children}
    </div>
  )
}
