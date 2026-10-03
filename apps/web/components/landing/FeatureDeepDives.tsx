"use client"

import { useRef, useState } from "react"
import { useLenis } from "lenis/react"
import { Braces, Eye, FileCode2, KeyRound, Network, Rows3, ShieldCheck, Sparkles, PencilLine, Zap, type LucideIcon } from "lucide-react"
import { AnimatePresence, motion, useMotionValue, useMotionValueEvent, useScroll, useSpring, useTransform } from "motion/react"

import { cn } from "@queryon/ui/lib/utils"

import { SectionLabel } from "./Brand"
import { FeatureScene } from "./FeatureScene"
import { ConnectionMock } from "./mocks/ConnectionMock"
import { DataGridMock } from "./mocks/DataGridMock"
import { SchemaMock } from "./mocks/SchemaMock"
import { SqlEditorMock } from "./mocks/SqlEditorMock"

const FEATURES = [
  {
    title: "Connect to any database, securely",
    description: "SSL encryption, SSH tunnelling, and passwords stored in your OS keychain — never in plain text.",
    tag: "Connect",
    visual: <ConnectionMock />,
    log: "tls verify-full · ssh tunnel up · 38 ms",
    chips: [
      { icon: ShieldCheck, label: "SSL verify-full" },
      { icon: Network, label: "SSH tunnel" },
      { icon: KeyRound, label: "OS keychain" },
    ],
  },
  {
    title: "Write SQL at full speed",
    description: "Syntax highlighting and autocomplete for your tables, so you can work quickly.",
    tag: "Query",
    visual: <SqlEditorMock />,
    log: "autocomplete · tables & columns loaded",
    chips: [
      { icon: Sparkles, label: "Schema-aware autocomplete" },
      { icon: Zap, label: "⌘ ↵ to run" },
      { icon: FileCode2, label: "Syntax highlighting" },
    ],
  },
  {
    title: "Easily view & edit data",
    description: "Browse and edit tables in a spreadsheet-like grid. Edit JSON cells with syntax checking, even when stored as TEXT.",
    tag: "Browse",
    visual: <DataGridMock compact height="260px" />,
    log: "users · 500 rows virtualized · json ✓",
    chips: [
      { icon: Rows3, label: "Virtualized grid" },
      { icon: Braces, label: "JSON validation" },
      { icon: PencilLine, label: "Inline cell editing" },
    ],
  },
  {
    title: "Schema editing with a preview",
    description:
      "Add tables, columns, indexes and constraints through a visual editor. Review the exact DDL before anything runs — no surprises on a live database.",
    tag: "Schema",
    visual: <SchemaMock />,
    log: "ddl preview · 1 statement · nothing runs until you say so",
    chips: [
      { icon: Eye, label: "Exact DDL preview" },
      { icon: ShieldCheck, label: "No surprises" },
      { icon: FileCode2, label: "Indexes & constraints" },
    ],
  },
]

const EASE = [0.16, 1, 0.3, 1] as const

function SectionIntro() {
  return (
    <div className="max-w-2xl">
      <SectionLabel>Features</SectionLabel>
      <h2 className="mt-5 text-[clamp(32px,4.4vw,56px)] leading-[1.04] font-semibold tracking-[-0.04em]">
        Simple to use. <span className="text-brand">Easy to love.</span>
      </h2>
      <p className="mt-4 max-w-xl text-[17px] text-muted-foreground">
        Not a scaled-down toy client, and not an enterprise tool with a decade of accumulated menus — just the parts you
        actually use, done well.
      </p>
    </div>
  )
}

const CHIP_SLOTS = ["-left-10 top-8", "-right-12 top-[42%]", "left-12 -bottom-6"]

interface StageChipProps {
  icon: LucideIcon
  label: string
  slot: number
}

function StageChip({ icon: Icon, label, slot }: StageChipProps) {
  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.6, y: 14, filter: "blur(6px)" }}
      animate={{ opacity: 1, scale: 1, y: 0, filter: "blur(0px)" }}
      exit={{ opacity: 0, scale: 0.85, y: -10, filter: "blur(4px)" }}
      transition={{ type: "spring", stiffness: 260, damping: 22, delay: 0.25 + slot * 0.09 }}
      style={{ z: 70 }}
      className={cn("pointer-events-none absolute z-10", CHIP_SLOTS[slot])}
    >
      <motion.span
        animate={{ y: [0, -6, 0] }}
        transition={{ duration: 4 + slot, repeat: Infinity, ease: "easeInOut", delay: slot * 0.6 }}
        className="flex items-center gap-2 rounded-full border bg-card/85 py-1.5 pr-3.5 pl-1.5 font-mono text-[11px] whitespace-nowrap text-foreground/85 shadow-[0_14px_40px_-14px_rgba(21,128,61,0.45)] backdrop-blur-md"
      >
        <span className="grid size-6 place-items-center rounded-full bg-brand-soft text-brand">
          <Icon className="size-3.5" />
        </span>
        {label}
      </motion.span>
    </motion.div>
  )
}

function TypedLog({ text }: { text: string }) {
  return (
    <motion.span
      key={text}
      initial="hidden"
      animate="shown"
      variants={{ shown: { transition: { staggerChildren: 0.014, delayChildren: 0.35 } } }}
      className="truncate"
      aria-label={text}
    >
      {Array.from(text).map((char, i) => (
        <motion.span key={i} aria-hidden variants={{ hidden: { opacity: 0 }, shown: { opacity: 1 } }}>
          {char}
        </motion.span>
      ))}
    </motion.span>
  )
}

interface FeatureStageProps {
  index: number
  immersive?: boolean
  onSelect?: (index: number) => void
}

function FeatureStage({ index, immersive, onSelect }: FeatureStageProps) {
  const feature = FEATURES[index]!
  const px = useMotionValue(0)
  const py = useMotionValue(0)
  const sx = useSpring(px, { stiffness: 120, damping: 18 })
  const sy = useSpring(py, { stiffness: 120, damping: 18 })
  const rotateY = useTransform(sx, [-0.5, 0.5], [-9, 9])
  const rotateX = useTransform(sy, [-0.5, 0.5], [7, -7])

  const track = (event: React.PointerEvent<HTMLDivElement>) => {
    if (!immersive) return
    const rect = event.currentTarget.getBoundingClientRect()
    px.set((event.clientX - rect.left) / rect.width - 0.5)
    py.set((event.clientY - rect.top) / rect.height - 0.5)
  }
  const reset = () => {
    px.set(0)
    py.set(0)
  }

  return (
    <div onPointerMove={track} onPointerLeave={reset} className={cn("relative isolate flex flex-col", immersive && "h-[min(620px,calc(100vh-140px))]")}>
      {immersive && (
        <div aria-hidden className="pointer-events-none absolute -inset-x-16 -inset-y-12 -z-10 mask-[radial-gradient(ellipse_at_center,black_40%,transparent_72%)]">
          <FeatureScene active={index} />
        </div>
      )}
      <div aria-hidden className="pointer-events-none absolute inset-x-[12%] top-1/2 -z-10 h-1/2 -translate-y-1/2 rounded-full bg-brand-bright/25 blur-[90px] dark:bg-brand-bright/12" />

      <div className="mx-auto flex w-full max-w-[540px] items-center justify-between">
        <AnimatePresence mode="wait" initial={false}>
          <motion.p
            key={feature.tag}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.3, ease: EASE }}
            className="flex items-center gap-2.5 font-mono text-xs tracking-[0.22em] text-brand uppercase"
          >
            <span className="text-muted-foreground/60">{String(index + 1).padStart(2, "0")}</span>
            <span className="h-px w-5 bg-brand/50" />
            {feature.tag}
          </motion.p>
        </AnimatePresence>
        {onSelect && (
          <div className="flex items-center gap-1.5">
            {FEATURES.map((f, i) => (
              <button key={f.tag} type="button" aria-label={f.title} onClick={() => onSelect(i)} className="grid h-5 place-items-center">
                <motion.span
                  animate={{ width: i === index ? 36 : 14, opacity: i === index ? 1 : 0.25 }}
                  transition={{ duration: 0.5, ease: EASE }}
                  className="block h-[3px] rounded-full bg-brand"
                />
              </button>
            ))}
          </div>
        )}
      </div>

      <div className={"flex flex-1 items-center justify-center py-6"} style={{ perspective: 1400 }}>
        <motion.div style={immersive ? { rotateX, rotateY, transformStyle: "preserve-3d" } : undefined} className="relative w-full max-w-[540px]">
          <AnimatePresence mode="wait" initial={false}>
            <motion.div
              key={index}
              initial={{ opacity: 0, y: 36, scale: 0.94, rotateX: 14, filter: "blur(8px)" }}
              animate={{ opacity: 1, y: 0, scale: 1, rotateX: 0, filter: "blur(0px)" }}
              exit={{ opacity: 0, y: -28, scale: 0.96, rotateX: -10, filter: "blur(8px)" }}
              transition={{ duration: 0.55, ease: EASE }}
              className="rounded-2xl shadow-[0_40px_100px_-30px_rgba(21,128,61,0.45)]"
            >
              {feature.visual}
            </motion.div>
          </AnimatePresence>
          {immersive && (
            <AnimatePresence>
              {feature.chips.map((chip, slot) => (
                <StageChip key={`${index}-${chip.label}`} icon={chip.icon} label={chip.label} slot={slot} />
              ))}
            </AnimatePresence>
          )}
        </motion.div>
      </div>

      <div className="mx-auto flex w-full max-w-[540px] items-center gap-2.5 font-mono text-[11px] text-muted-foreground">
        <span className="size-1.5 shrink-0 animate-pulse-dot rounded-full bg-brand-bright" />
        <span className="shrink-0 text-brand">queryon ›</span>
        <TypedLog text={feature.log} />
      </div>
    </div>
  )
}

function PinnedFeatures() {
  const ref = useRef<HTMLDivElement>(null)
  const [active, setActive] = useState(0)
  const { scrollYProgress } = useScroll({ target: ref, offset: ["start start", "end end"] })
  const bar = useTransform(scrollYProgress, [0, 1], ["0%", "100%"])
  useMotionValueEvent(scrollYProgress, "change", (p) => setActive(Math.min(FEATURES.length - 1, Math.floor(p * FEATURES.length))))

  const lenis = useLenis()

  const goTo = (i: number) => {
    const el = ref.current
    if (!el) return
    const travel = el.offsetHeight - window.innerHeight
    const top = el.getBoundingClientRect().top + window.scrollY + travel * ((i + 0.5) / FEATURES.length)
    if (lenis) lenis.scrollTo(top, { duration: 1.1 })
    else window.scrollTo({ top, behavior: "smooth" })
  }

  return (
    <div ref={ref} className="relative hidden lg:block" style={{ height: `${FEATURES.length * 85 + 30}vh` }}>
      <div className="sticky top-0 flex h-screen items-center">
        <div className="grid w-full grid-cols-[0.85fr_1.15fr] items-center gap-12 xl:gap-16">
          <div>
            <SectionIntro />
            <ol className="mt-10 flex flex-col gap-1">
              {FEATURES.map((f, i) => {
                const on = i === active
                return (
                  <li key={f.title}>
                    <button
                      type="button"
                      onClick={() => goTo(i)}
                      aria-current={on ? "step" : undefined}
                      className={cn("w-full rounded-2xl border px-5 py-4 text-left transition-colors duration-500", on ? "border-border bg-card shadow-sm" : "border-transparent hover:bg-muted/50")}
                    >
                      <span className="flex items-center gap-3">
                        <span className={cn("font-mono text-xs transition-colors", on ? "text-brand" : "text-muted-foreground/60")}>{String(i + 1).padStart(2, "0")}</span>
                        <span className={cn("text-lg font-medium tracking-tight transition-colors", on ? "text-foreground" : "text-muted-foreground")}>{f.title}</span>
                      </span>
                      <motion.span initial={false} animate={{ height: on ? "auto" : 0, opacity: on ? 1 : 0 }} transition={{ duration: 0.45, ease: EASE }} className="block overflow-hidden">
                        <span className="block pt-2 pl-8 text-[15px] text-muted-foreground">{f.description}</span>
                      </motion.span>
                    </button>
                  </li>
                )
              })}
            </ol>
            <div className="mt-6 h-px bg-border">
              <motion.div style={{ width: bar }} className="h-px bg-brand" />
            </div>
          </div>

          <FeatureStage index={active} immersive onSelect={goTo} />
        </div>
      </div>
    </div>
  )
}

function StackedFeatures() {
  return (
    <div className="lg:hidden">
      <SectionIntro />
      <div className="mt-12 flex flex-col gap-14">
        {FEATURES.map((f, i) => (
          <motion.div key={f.title} initial={{ opacity: 0, y: 30 }} whileInView={{ opacity: 1, y: 0 }} viewport={{ once: true, margin: "-10%" }} transition={{ duration: 0.7, ease: EASE }}>
            <span className="font-mono text-xs text-brand">{String(i + 1).padStart(2, "0")}</span>
            <h3 className="mt-2 text-2xl font-medium tracking-tight">{f.title}</h3>
            <p className="mt-2 text-muted-foreground">{f.description}</p>
            <div className="mt-6">
              <FeatureStage index={i} />
            </div>
          </motion.div>
        ))}
      </div>
    </div>
  )
}

export function FeatureDeepDives() {
  return (
    <section id="features" className="mx-auto max-w-7xl scroll-mt-24 px-5 sm:px-6">
      <PinnedFeatures />
      <StackedFeatures />
    </section>
  )
}
