"use client"

import { useRef } from "react"
import {
  ChevronRight,
  Database,
  Download,
  KeyRound,
  Keyboard,
  Laptop,
  ShieldCheck,
  Waypoints,
} from "lucide-react"

import { cn } from "@queryon/ui/lib/utils"

const CAROUSEL_FEATURES = [
  {
    title: "Pop out any tab",
    description: "Drag a table or query tab into its own window, then dock it back.",
    icon: Laptop,
    dark: true,
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

export function MoreFeaturesCarousel() {
  const scrollerRef = useRef<HTMLDivElement>(null)

  function scrollNext() {
    scrollerRef.current?.scrollBy({ left: 320, behavior: "smooth" })
  }

  return (
    <section className="mx-auto max-w-7xl px-6">
      <p className="mb-6 text-2xl font-medium text-balance">
        <span className="bg-foreground px-1.5 text-background">More to love.</span>{" "}
        <span className="text-muted-foreground">Everything else Queryon does well.</span>
      </p>

      <div className="relative">
        <div
          ref={scrollerRef}
          className="flex snap-x snap-mandatory gap-4 overflow-x-auto pb-2 [mask-image:linear-gradient(to_right,transparent,black_64px,black_calc(100%-64px),transparent)] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
        >
          {CAROUSEL_FEATURES.map(({ title, description, icon: Icon, dark }) => (
            <div
              key={title}
              className={cn(
                "flex w-64 shrink-0 snap-start flex-col justify-between gap-10 rounded-2xl border p-6",
                dark ? "bg-foreground text-background" : "bg-card"
              )}
            >
              <div>
                <h3 className="text-lg font-medium">{title}</h3>
                <p
                  className={cn(
                    "mt-1.5 text-sm text-balance",
                    dark ? "text-background/70" : "text-muted-foreground"
                  )}
                >
                  {description}
                </p>
              </div>
              <Icon className={cn("size-9", dark ? "text-background/90" : "text-foreground/70")} />
            </div>
          ))}
        </div>

        <button
          type="button"
          onClick={scrollNext}
          aria-label="Scroll to next feature"
          className="absolute top-1/2 right-0 hidden size-10 -translate-y-1/2 translate-x-1/2 items-center justify-center rounded-full border bg-background shadow-md transition-colors hover:bg-muted sm:flex"
        >
          <ChevronRight className="size-5" />
        </button>
      </div>
    </section>
  )
}
