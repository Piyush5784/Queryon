"use client"

import { X } from "lucide-react"
import { motion, type Variants } from "motion/react"

import { cn } from "@queryon/ui/lib/utils"

import { LogoMark, SectionLabel } from "./Brand"

type Tone = "yes" | "partial" | "no"

interface Verdict {
  tone: Tone
  note?: string
}

interface ComparisonRow {
  label: string
  hint: string
  queryon: Verdict
  cells: Record<Tool, Verdict>
}

const TOOLS = ["DBeaver", "DataGrip", "TablePlus", "pgAdmin", "Navicat", "Beekeeper Studio"] as const
type Tool = (typeof TOOLS)[number]

const ROWS: ComparisonRow[] = [
  {
    label: "Cold start time",
    hint: "Launch to a usable window",
    queryon: { tone: "yes", note: "~0.2s" },
    cells: {
      DBeaver: { tone: "no", note: "3–12s" },
      DataGrip: { tone: "no", note: "10–30s" },
      TablePlus: { tone: "yes", note: "<1s" },
      pgAdmin: { tone: "no", note: "5–30s" },
      Navicat: { tone: "partial", note: "1–3s" },
      "Beekeeper Studio": { tone: "partial", note: "2–4s" },
    },
  },
  {
    label: "Free, no paid tier",
    hint: "Every feature, no upsell",
    queryon: { tone: "yes", note: "Free" },
    cells: {
      DBeaver: { tone: "partial", note: "Freemium" },
      DataGrip: { tone: "no", note: "Paid" },
      TablePlus: { tone: "partial", note: "Freemium" },
      pgAdmin: { tone: "yes", note: "Free" },
      Navicat: { tone: "no", note: "Paid" },
      "Beekeeper Studio": { tone: "partial", note: "Freemium" },
    },
  },
  {
    label: "Built for writing queries",
    hint: "Not DBA administration",
    queryon: { tone: "yes" },
    cells: {
      DBeaver: { tone: "yes" },
      DataGrip: { tone: "yes" },
      TablePlus: { tone: "yes" },
      pgAdmin: { tone: "no", note: "Admin-first" },
      Navicat: { tone: "partial", note: "Admin-heavy" },
      "Beekeeper Studio": { tone: "yes" },
    },
  },
  {
    label: "Native desktop app",
    hint: "No JVM, no bundled Chromium",
    queryon: { tone: "yes", note: "Rust core" },
    cells: {
      DBeaver: { tone: "no", note: "JVM" },
      DataGrip: { tone: "no", note: "JVM" },
      TablePlus: { tone: "yes" },
      pgAdmin: { tone: "no", note: "Electron" },
      Navicat: { tone: "yes" },
      "Beekeeper Studio": { tone: "no", note: "Electron" },
    },
  },
  {
    label: "Multiple SQL engines",
    hint: "One app for every database",
    queryon: { tone: "yes" },
    cells: {
      DBeaver: { tone: "yes" },
      DataGrip: { tone: "yes" },
      TablePlus: { tone: "yes" },
      pgAdmin: { tone: "no", note: "Postgres only" },
      Navicat: { tone: "yes" },
      "Beekeeper Studio": { tone: "yes" },
    },
  },
]

const EASE = [0.16, 1, 0.3, 1] as const
const TEMPLATE = "minmax(230px,1.5fr) minmax(132px,1fr) repeat(6,minmax(108px,1fr))"

const reveal: Variants = {
  hidden: { opacity: 0, y: 14 },
  shown: (delay: number) => ({ opacity: 1, y: 0, transition: { duration: 0.55, ease: EASE, delay } }),
}

const pop: Variants = {
  hidden: { scale: 0, opacity: 0 },
  shown: (delay: number) => ({ scale: 1, opacity: 1, transition: { type: "spring", stiffness: 380, damping: 18, delay } }),
}

const draw: Variants = {
  hidden: { pathLength: 0 },
  shown: (delay: number) => ({ pathLength: 1, transition: { duration: 0.45, ease: "easeOut", delay: delay + 0.15 } }),
}

function CheckMark({ delay, className }: { delay: number; className?: string }) {
  return (
    <svg viewBox="0 0 24 24" fill="none" strokeWidth={3} strokeLinecap="round" strokeLinejoin="round" stroke="currentColor" className={className}>
      <motion.path d="M5 12.5l4.5 4.5L19 7.5" variants={draw} custom={delay} />
    </svg>
  )
}

function VerdictMark({ verdict, delay, highlight }: { verdict: Verdict; delay: number; highlight?: boolean }) {
  const { tone, note } = verdict
  return (
    <div className="flex flex-col items-center gap-1.5">
      <motion.span
        variants={pop}
        custom={delay}
        className={cn(
          "grid place-items-center rounded-full",
          highlight
            ? "size-8 bg-brand-bright text-[#04140a] shadow-[0_0_24px_-2px_rgba(74,222,128,0.65)]"
            : tone === "yes"
              ? "size-6 bg-brand-soft text-brand"
              : tone === "partial"
                ? "size-6 bg-amber-500/12 text-amber-600 dark:text-amber-400"
                : "size-6 bg-muted text-muted-foreground/60"
        )}
      >
        {tone === "yes" ? (
          <CheckMark delay={delay} className={highlight ? "size-4" : "size-3.5"} />
        ) : tone === "partial" ? (
          <svg viewBox="0 0 24 24" className="size-3.5">
            <circle cx="12" cy="12" r="7.5" fill="none" stroke="currentColor" strokeWidth="2.5" />
            <path d="M12 4.5a7.5 7.5 0 0 1 0 15z" fill="currentColor" />
          </svg>
        ) : (
          <X className="size-3.5" strokeWidth={2.5} />
        )}
      </motion.span>
      {note && (
        <span
          className={cn(
            "font-mono text-[11px] whitespace-nowrap",
            highlight
              ? "font-medium text-brand-bright"
              : tone === "yes"
                ? "text-brand"
                : tone === "partial"
                  ? "text-amber-700 dark:text-amber-400/90"
                  : "text-muted-foreground"
          )}
        >
          {note}
        </span>
      )}
    </div>
  )
}

export function CompetitorComparison() {
  const rowCount = ROWS.length + 1

  return (
    <section className="mx-auto max-w-7xl px-5 sm:px-6">
      <div className="flex flex-col items-start justify-between gap-4 md:flex-row md:items-end">
        <div>
          <SectionLabel>Compare</SectionLabel>
          <h2 className="mt-4 text-[clamp(28px,3.4vw,44px)] font-semibold tracking-[-0.035em]">
            How Queryon <span className="text-brand">stacks up.</span>
          </h2>
        </div>
        <p className="max-w-sm text-muted-foreground">An honest look at where Queryon stands next to the tools you already know.</p>
      </div>

      <motion.div
        initial="hidden"
        whileInView="shown"
        viewport={{ once: true, margin: "-12%" }}
        variants={reveal}
        custom={0}
        className="mt-10 overflow-hidden rounded-[28px] border bg-card shadow-[0_30px_80px_-50px_rgba(5,46,22,0.35)]"
      >
        <div className="overflow-x-auto">
          <div role="table" aria-label="Queryon compared with other database clients" className="relative grid min-w-250 p-2" style={{ gridTemplateColumns: TEMPLATE }}>
            <motion.div
              aria-hidden
              variants={{
                hidden: { opacity: 0, scaleY: 0.4 },
                shown: { opacity: 1, scaleY: 1, transition: { duration: 0.8, ease: EASE, delay: 0.1 } },
              }}
              style={{ gridColumn: 2, gridRow: `1 / span ${rowCount}`, originY: 0 }}
              className="relative overflow-hidden rounded-[22px] bg-ink ring-1 ring-brand-bright/25"
            >
              <div className="console-lines absolute inset-0 opacity-60" />
              <div className="absolute inset-x-0 top-0 h-40 bg-[radial-gradient(80%_100%_at_50%_0%,rgba(74,222,128,0.28),transparent_70%)]" />
              <motion.div
                animate={{ y: ["-30%", "130%"] }}
                transition={{ duration: 5, repeat: Infinity, ease: "linear", repeatDelay: 1.5 }}
                className="absolute inset-x-0 h-24 bg-linear-to-b from-transparent via-brand-bright/10 to-transparent"
              />
            </motion.div>

            <div role="row" className="contents">
              <div role="columnheader" style={{ gridColumn: 1, gridRow: 1 }} className="sticky left-0 z-20 flex items-end bg-card px-5 pt-6 pb-5 font-mono text-[11px] tracking-[0.18em] text-muted-foreground uppercase">
                Capability
              </div>
              <div role="columnheader" style={{ gridColumn: 2, gridRow: 1 }} className="relative z-10 flex flex-col items-center justify-end gap-1 px-3 pt-6 pb-5 text-white">
                <span className="flex items-center gap-2 text-[15px] font-semibold">
                  <LogoMark className="size-5" />
                  Queryon
                </span>
              </div>
              {TOOLS.map((tool, i) => (
                <motion.div
                  key={tool}
                  role="columnheader"
                  variants={reveal}
                  custom={0.15 + i * 0.04}
                  style={{ gridColumn: i + 3, gridRow: 1 }}
                  className="flex items-end justify-center px-3 pt-6 pb-5 text-center text-sm font-medium text-muted-foreground"
                >
                  {tool}
                </motion.div>
              ))}
            </div>

            {ROWS.map((row, r) => {
              const gridRow = r + 2
              const base = 0.25 + r * 0.08
              return (
                <div key={row.label} role="row" className="group contents">
                  <motion.div
                    role="rowheader"
                    variants={reveal}
                    custom={base}
                    style={{ gridColumn: 1, gridRow }}
                    className="sticky left-0 z-20 flex flex-col justify-center border-t bg-card px-5 py-4 transition-colors group-hover:bg-muted"
                  >
                    <span className="text-[15px] font-medium tracking-tight">{row.label}</span>
                    <span className="mt-0.5 text-xs text-muted-foreground">{row.hint}</span>
                  </motion.div>
                  <div role="cell" style={{ gridColumn: 2, gridRow }} className="relative z-10 grid place-items-center border-t border-white/10 px-3 py-4 transition-colors group-hover:bg-white/4">
                    <VerdictMark verdict={row.queryon} delay={base + 0.1} highlight />
                  </div>
                  {TOOLS.map((tool, c) => (
                    <motion.div
                      key={tool}
                      role="cell"
                      variants={reveal}
                      custom={base + 0.05 + c * 0.03}
                      style={{ gridColumn: c + 3, gridRow }}
                      className="grid place-items-center border-t px-3 py-4 transition-colors group-hover:bg-muted"
                    >
                      <VerdictMark verdict={row.cells[tool]} delay={base + 0.1 + c * 0.03} />
                    </motion.div>
                  ))}
                </div>
              )
            })}
          </div>
        </div>
      </motion.div>

      <div className="mt-5 flex flex-col items-center gap-3">
        <div className="flex flex-wrap items-center justify-center gap-x-5 gap-y-2 text-xs text-muted-foreground">
          <span className="flex items-center gap-1.5">
            <span className="size-2 rounded-full bg-brand" /> Yes
          </span>
          <span className="flex items-center gap-1.5">
            <span className="size-2 rounded-full bg-amber-500" /> Partly / with limits
          </span>
          <span className="flex items-center gap-1.5">
            <span className="size-2 rounded-full bg-muted-foreground/40" /> No
          </span>
        </div>
        <p className="max-w-3xl text-center text-xs text-muted-foreground">
          Startup times are approximate. Queryon&apos;s was measured on an unoptimized debug build on Linux — a release build should
          only be faster. Competitor figures are commonly reported ranges, not benchmarks we ran ourselves.
        </p>
      </div>
    </section>
  )
}
