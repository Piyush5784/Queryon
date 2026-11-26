"use client";

import {
  Database,
  Download,
  KeyRound,
  Keyboard,
  Laptop,
  Waypoints,
} from "lucide-react";
import { motion } from "motion/react";

import { cn } from "@queryon/ui/lib/utils";

import { SectionLabel } from "./Brand";

const FEATURES = [
  {
    title: "Pop out any tab",
    description:
      "Drag a table or query tab into its own window, then dock it back.",
    icon: Laptop,
    className: "md:col-span-2",
    tone: "dark",
  },
  {
    title: "Every engine, one app",
    description:
      "Postgres, MySQL, SQL Server, SQLite, ClickHouse and more.",
    icon: Database,
  },
  {
    title: "Credentials stay local",
    description:
      "Passwords go through your OS keychain, never a third-party server.",
    icon: KeyRound,
    tone: "brand",
  },
  {
    title: "SSH tunnelling built in",
    description:
      "Reach databases behind a bastion host, no separate tunnel needed.",
    icon: Waypoints,
  },
  {
    title: "Export anything",
    description:
      "CSV, JSON or SQL — export a query, a table or a whole schema.",
    icon: Download,
  },
  {
    title: "Keyboard-first",
    description:
      "Query, run, save and switch tabs — all without touching the mouse.",
    icon: Keyboard,
    className: "md:col-span-2",
    keys: ["⌘", "↵"],
  },
];

export function MoreFeaturesCarousel() {
  return (
    <section className="mx-auto max-w-7xl px-5 sm:px-6">
      <SectionLabel>More to love</SectionLabel>
      <h2 className="mt-4 max-w-2xl text-[clamp(28px,3.4vw,44px)] font-semibold tracking-[-0.035em]">
        Everything else Queryon <span className="text-brand">does well.</span>
      </h2>

      <div className="mt-10 grid grid-cols-1 gap-3 md:grid-cols-4">
        {FEATURES.map(
          ({ title, description, icon: Icon, className, tone, keys }, i) => (
            <motion.div
              key={title}
              initial={{ opacity: 0, y: 24 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: "-8%" }}
              transition={{
                duration: 0.6,
                ease: [0.16, 1, 0.3, 1],
                delay: (i % 4) * 0.07,
              }}
              className={cn(
                "group relative flex min-h-52 flex-col justify-between overflow-hidden rounded-[24px] border p-6 transition-shadow duration-500 hover:shadow-[0_30px_60px_-30px_rgba(21,128,61,0.45)]",
                tone === "dark" &&
                  "console-lines border-transparent bg-ink text-white",
                tone === "brand" &&
                  "border-transparent bg-brand-bright text-[#04140a]",
                !tone && "bg-card",
                className,
              )}
            >
              <span
                className={cn(
                  "grid size-11 place-items-center rounded-2xl transition-transform duration-500 group-hover:-rotate-6",
                  tone === "dark"
                    ? "bg-brand-bright text-[#04140a]"
                    : tone === "brand"
                      ? "bg-ink text-brand-bright"
                      : "bg-brand-soft text-brand",
                )}
              >
                <Icon className="size-5" />
              </span>
              <div className="mt-8">
                {keys && (
                  <div className="mb-3 flex gap-1.5">
                    {keys.map((k) => (
                      <kbd
                        key={k}
                        className="grid size-9 place-items-center rounded-lg border-b-2 bg-muted font-mono text-sm"
                      >
                        {k}
                      </kbd>
                    ))}
                  </div>
                )}
                <h3 className="text-lg font-medium tracking-tight">{title}</h3>
                <p
                  className={cn(
                    "mt-1.5 text-sm",
                    tone ? "opacity-70" : "text-muted-foreground",
                  )}
                >
                  {description}
                </p>
              </div>
            </motion.div>
          ),
        )}
      </div>
    </section>
  );
}
