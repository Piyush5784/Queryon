import {
  Database,
  Download,
  KeyRound,
  Keyboard,
  Laptop,
  ShieldCheck,
  Waypoints,
} from "lucide-react"

const CAROUSEL_FEATURES = [
  {
    title: "Pop out any tab",
    description: "Drag a table or query tab into its own window, then dock it back.",
    icon: Laptop,
  },
  {
    title: "Every engine, one app",
    description: "Postgres, MySQL, SQL Server, SQLite, DuckDB, ClickHouse, and more.",
    icon: Database,
  },
  {
    title: "Credentials stay local",
    description: "Passwords go through your OS keychain, never a third-party server.",
    icon: KeyRound,
  },
  {
    title: "SSH tunnelling built in",
    description: "Reach databases behind a bastion host, no separate tunnel needed.",
    icon: Waypoints,
  },
  {
    title: "Export anything",
    description: "CSV, JSON, or SQL — export a query, table, or whole schema.",
    icon: Download,
  },
  {
    title: "Keyboard-first",
    description: "Query, run, save, switch tabs — all without touching the mouse.",
    icon: Keyboard,
  },
  {
    title: "Browse documents too",
    description: "MongoDB collections get their own browser, not rows shoehorned into a grid.",
    icon: ShieldCheck,
  },
]

const LOOPED_FEATURES = [...CAROUSEL_FEATURES, ...CAROUSEL_FEATURES]

export function MoreFeaturesCarousel() {
  return (
    <section className="mx-auto max-w-7xl px-6">
      <p className="mb-6 text-2xl font-medium text-balance">
        <span className="bg-foreground px-1.5 text-background">More to love.</span>{" "}
        <span className="text-muted-foreground">Everything else Queryon does well.</span>
      </p>

      <div className="group relative overflow-hidden [mask-image:linear-gradient(to_right,transparent,black_64px,black_calc(100%-64px),transparent)]">
        <div className="flex w-max animate-[marquee-rtl_36s_linear_infinite] gap-4 pb-2 group-hover:[animation-play-state:paused]">
          {LOOPED_FEATURES.map(({ title, description, icon: Icon }, i) => (
            <div
              key={`${title}-${i}`}
              className="flex w-64 shrink-0 flex-col justify-between gap-10 rounded-2xl border bg-card p-6"
            >
              <div>
                <h3 className="text-lg font-medium">{title}</h3>
                <p className="mt-1.5 text-sm text-balance text-muted-foreground">{description}</p>
              </div>
              <Icon className="size-9 text-foreground/70" />
            </div>
          ))}
        </div>
      </div>
    </section>
  )
}
