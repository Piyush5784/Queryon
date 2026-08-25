const BENTO_FEATURES = [
  {
    title: "Native, not Electron",
    description: "Built on Tauri — a fraction of the install size and memory of Electron clients.",
    icon: "size",
  },
  {
    title: "Every engine, one app",
    description:
      "Postgres, MySQL, SQL Server, SQLite, DuckDB, ClickHouse, and more — no separate tools.",
    icon: "engines",
  },
  {
    title: "Credentials stay local",
    description: "Passwords go through your OS keychain, never through a third-party server.",
    icon: "lock",
  },
  {
    title: "SSH tunnelling built in",
    description: "Reach databases behind a bastion host without a separate terminal tunnel.",
    icon: "tunnel",
  },
  {
    title: "Export anything",
    description: "CSV, JSON, or SQL — export a query, a table, or a whole schema in a couple clicks.",
    icon: "export",
  },
  {
    title: "Keyboard-first",
    description:
      "New query, run, save, cancel, switch tabs — all reachable without touching the mouse.",
    icon: "keys",
  },
  {
    title: "Cross-platform",
    description: "Windows, macOS, and Linux builds from the same codebase, updated in the background.",
    icon: "platforms",
  },
]

function FeatureCard({ title, description }: { title: string; description: string }) {
  return (
    <div className="flex flex-col gap-6 rounded-xl border bg-card p-6 text-left md:p-8">
      <div className="flex h-24 items-center justify-center rounded-lg border bg-muted/30">
        <div className="size-8 rounded-md border-2 border-dashed border-muted-foreground/30" />
      </div>
      <div className="flex flex-col gap-1.5">
        <h3 className="text-lg font-medium">{title}</h3>
        <p className="text-sm text-muted-foreground">{description}</p>
      </div>
    </div>
  )
}

export function FeatureGrid() {
  return (
    <div className="mx-auto max-w-[1200px] px-6 text-center">
      <h2 className="mb-3 text-3xl font-medium tracking-tight text-balance sm:text-4xl">
        Everything you'd expect. Nothing you don't.
      </h2>
      <p className="mx-auto max-w-xl text-lg text-muted-foreground text-balance">
        A short list of the things that make Queryon feel different from the moment you install
        it.
      </p>

      <div className="mt-12 flex flex-col gap-4">
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
          {BENTO_FEATURES.slice(0, 2).map((f) => (
            <FeatureCard key={f.title} {...f} />
          ))}
        </div>
        <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
          {BENTO_FEATURES.slice(2, 5).map((f) => (
            <FeatureCard key={f.title} {...f} />
          ))}
        </div>
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
          {BENTO_FEATURES.slice(5, 7).map((f) => (
            <FeatureCard key={f.title} {...f} />
          ))}
        </div>
      </div>
    </div>
  )
}
