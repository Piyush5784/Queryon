"use client"

import { useEffect, useState } from "react"
import Link from "next/link"
import { ArrowRight, GitBranch, Table2, Terminal, Waypoints } from "lucide-react"
import { AnimatePresence, motion } from "motion/react"

import { buttonVariants } from "@queryon/ui/components/button"
import { cn } from "@queryon/ui/lib/utils"

import { HeroScene } from "./HeroScene"
import { DataGridMock } from "./mocks/DataGridMock"

const HERO_MODES = [
  {
    id: "browse" as const,
    label: "Browse",
    icon: Table2,
    color: "text-blue-600 dark:text-blue-400",
    title: "Browse every table.\nEdit rows in place.",
    tagline: "with Browse mode…",
  },
  {
    id: "query" as const,
    label: "Query",
    icon: Terminal,
    color: "text-emerald-600 dark:text-emerald-400",
    title: "Write raw SQL.\nSee results instantly.",
    tagline: "with Query mode…",
  },
  {
    id: "schema" as const,
    label: "Schema",
    icon: Waypoints,
    color: "text-amber-600 dark:text-amber-400",
    title: "Edit schema visually, DDL first.",
    tagline: "with Schema mode…",
  },
]

const MODE_CYCLE_INTERVAL = 3500

function HeroTitle({ text }: { text: string }) {
  const lines = text.split("\n")
  let letterIndex = -1

  return (
    <>
      {lines.map((line, lineIndex) => (
        <span key={`${line}:${lineIndex}`} className="block">
          {line.split("").map((char, i) => {
            letterIndex += 1
            if (char === " ") {
              return (
                <span key={`${char}:${i}`} className="inline-block">
                  &nbsp;
                </span>
              )
            }
            return (
              <motion.span
                key={`${char}:${i}`}
                className="inline-block"
                initial={{ opacity: 0, y: "0.4em", filter: "blur(6px)" }}
                animate={{ opacity: 1, y: "0em", filter: "blur(0px)" }}
                transition={{ duration: 0.34, delay: letterIndex * 0.012, ease: "easeOut" }}
              >
                {char}
              </motion.span>
            )
          })}
        </span>
      ))}
    </>
  )
}

export function Hero() {
  const [activeModeIndex, setActiveModeIndex] = useState(0)
  const [interacted, setInteracted] = useState(false)
  const activeMode = HERO_MODES[activeModeIndex] ?? HERO_MODES[0]!

  useEffect(() => {
    if (interacted) return
    const interval = setInterval(() => {
      setActiveModeIndex((prev) => (prev + 1) % HERO_MODES.length)
    }, MODE_CYCLE_INTERVAL)
    return () => clearInterval(interval)
  }, [interacted])

  return (
    <section className="relative overflow-hidden">
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 -z-10 [mask-image:linear-gradient(to_bottom,transparent,black_15%,black_85%,transparent)]"
      >
        <div className="absolute inset-0 bg-[linear-gradient(to_right,var(--border)_1px,transparent_1px),linear-gradient(to_bottom,var(--border)_1px,transparent_1px)] bg-[size:56px_56px] opacity-40" />
        <HeroScene />
      </div>

      <div className="mx-[150px] grid w-full items-start px-6 pt-20 pb-20 lg:grid-cols-[minmax(0,0.85fr)_minmax(0,1.15fr)] lg:gap-8 lg:pt-28">
        <div className="flex flex-col items-start gap-5 text-left">
          <a
            href="https://github.com/Piyush5784/Queryon"
            className="group inline-flex items-center gap-2 rounded-lg border bg-card px-3 py-1 text-xs text-muted-foreground shadow-sm transition-colors hover:border-foreground/20 hover:text-foreground"
          >
            <GitBranch className="size-3.5" />
            Free & open source on GitHub
            <ArrowRight className="size-3 transition-transform group-hover:translate-x-0.5" />
          </a>

          <div className="inline-flex gap-1 rounded-lg border bg-muted/40 p-1">
            {HERO_MODES.map((mode, index) => {
              const isActive = index === activeModeIndex
              const Icon = mode.icon
              return (
                <button
                  key={mode.id}
                  type="button"
                  onClick={() => {
                    setInteracted(true)
                    setActiveModeIndex(index)
                  }}
                  className="relative flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-sm font-medium"
                >
                  {isActive && (
                    <motion.span
                      layoutId="heroModeHighlight"
                      className="absolute inset-0 rounded-lg border bg-card shadow-sm"
                      transition={{ type: "spring", stiffness: 400, damping: 32 }}
                    />
                  )}
                  <Icon
                    className={cn(
                      "relative z-10 size-3.5 transition-colors",
                      isActive ? mode.color : "text-muted-foreground"
                    )}
                  />
                  <span
                    className={cn(
                      "relative z-10 transition-colors",
                      isActive ? "text-foreground" : "text-muted-foreground"
                    )}
                  >
                    {mode.label}
                  </span>
                </button>
              )
            })}
          </div>

          <div className="h-5">
            <AnimatePresence mode="wait" initial={false}>
              <motion.span
                key={activeMode.id}
                className={cn("block text-sm font-medium italic", activeMode.color)}
                initial={{ opacity: 0, y: 6 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -6 }}
                transition={{ duration: 0.25, ease: "easeOut" }}
              >
                {activeMode.tagline}
              </motion.span>
            </AnimatePresence>
          </div>

          <h1 className="relative flex min-h-[5.5rem] flex-col justify-center text-4xl font-medium tracking-tight text-balance sm:min-h-[8rem] sm:text-6xl">
            <AnimatePresence mode="wait" initial={false}>
              <motion.span
                key={activeMode.id}
                exit={{ opacity: 0, y: -16, filter: "blur(4px)", transition: { duration: 0.2 } }}
              >
                <HeroTitle text={activeMode.title} />
              </motion.span>
            </AnimatePresence>
          </h1>

          <p className="max-w-xl text-lg text-muted-foreground text-balance">
            A fast, native desktop client for Postgres, MySQL, SQL Server, ClickHouse, DuckDB, and
            a dozen more — browse schemas, run queries, and edit data without switching tools.
          </p>

          <div className="flex flex-wrap items-center gap-3 pt-2">
            <Link href="/docs/install" className={cn(buttonVariants({ size: "lg" }), "gap-2")}>
              Download for free
              <ArrowRight className="size-4" />
            </Link>
            <a
              href="https://github.com/Piyush5784/Queryon"
              className={buttonVariants({ size: "lg", variant: "outline" })}
            >
              View on GitHub
            </a>
          </div>
          <p className="text-xs text-muted-foreground">
            Windows, macOS, and Linux · No account required
          </p>
        </div>

        <div className="lg:-mt-15 lg:w-full">
          <DataGridMock fixedHeight />
        </div>
      </div>
    </section>
  )
}
