"use client";

import { useEffect, useRef, useState } from "react";
import dynamic from "next/dynamic";
import {
  Activity,
  Plug,
  ShieldCheck,
  Table2,
  Terminal,
  Waypoints,
} from "lucide-react";
import { AnimatePresence, motion, useScroll, useTransform } from "motion/react";

import { cn } from "@queryon/ui/lib/utils";

import { CtaButton, SectionLabel } from "./Brand";
import { DataGridMock } from "./mocks/DataGridMock";

const HeroScene = dynamic(
  () => import("./HeroScene").then((m) => m.HeroScene),
  { ssr: false },
);

const HERO_MODES = [
  {
    id: "browse",
    label: "Browse",
    icon: Table2,
    title: ["Browse every table.", "Edit rows in place."],
  },
  {
    id: "query",
    label: "Query",
    icon: Terminal,
    title: ["Write raw SQL.", "See results instantly."],
  },
  {
    id: "schema",
    label: "Schema",
    icon: Waypoints,
    title: ["Edit schema visually,", "DDL first."],
  },
] as const;

const STATUS_CARDS = [
  {
    icon: Plug,
    title: "Connected",
    detail: "Postgres 16 · devdb",
    className: "left-4 top-6 sm:-left-6 sm:top-12",
  },
  {
    icon: Activity,
    title: "Query ran",
    detail: "4 rows · 12 ms",
    className: "right-4 top-1/2 sm:-right-8",
    dark: true,
  },
  {
    icon: ShieldCheck,
    title: "SSH tunnel up",
    detail: "bastion.prod · keychain",
    className: "bottom-6 left-6 sm:bottom-10 sm:left-10",
  },
];

const MODE_CYCLE_INTERVAL = 3800;
const EASE = [0.16, 1, 0.3, 1] as const;

function AnimatedLine({ text, delay }: { text: string; delay: number }) {
  return (
    <span className="block">
      {text.split(" ").map((word, i, words) => (
        <span
          key={`${word}-${i}`}
          className="inline-block overflow-hidden pb-[0.08em] align-top"
        >
          <motion.span
            className="inline-block"
            initial={{ y: "105%" }}
            animate={{ y: "0%" }}
            transition={{ duration: 0.7, ease: EASE, delay: delay + i * 0.05 }}
          >
            {word}
            {i < words.length - 1 && " "}
          </motion.span>
        </span>
      ))}
    </span>
  );
}

export function Hero() {
  const [activeIndex, setActiveIndex] = useState(0);
  const [interacted, setInteracted] = useState(false);
  const active = HERO_MODES[activeIndex] ?? HERO_MODES[0];

  useEffect(() => {
    if (interacted) return;
    const interval = setInterval(
      () => setActiveIndex((prev) => (prev + 1) % HERO_MODES.length),
      MODE_CYCLE_INTERVAL,
    );
    return () => clearInterval(interval);
  }, [interacted]);

  const shotRef = useRef<HTMLDivElement>(null);
  const { scrollYProgress } = useScroll({
    target: shotRef,
    offset: ["start end", "center center"],
  });
  const rotateX = useTransform(scrollYProgress, [0, 1], [22, 0]);
  const scale = useTransform(scrollYProgress, [0, 1], [0.9, 1]);
  const shotY = useTransform(scrollYProgress, [0, 1], [60, 0]);

  return (
    <section className="relative overflow-hidden">
      <div
        aria-hidden
        className="grid-lines pointer-events-none absolute inset-0 -z-10 [mask-image:radial-gradient(70%_60%_at_50%_30%,black,transparent)]"
      />
      <div
        aria-hidden
        className="pointer-events-none absolute -top-40 left-1/2 -z-10 h-[520px] w-[900px] -translate-x-1/2 rounded-full bg-brand-bright/20 blur-[140px] dark:bg-brand-bright/10"
      />

      <div className="mx-auto grid max-w-7xl items-center gap-12 px-5 pt-14 pb-10 sm:px-6 lg:grid-cols-[1.05fr_1fr] lg:gap-10 lg:pt-20">
        <div className="flex flex-col items-start">
          <SectionLabel>One client for every database</SectionLabel>

          <div
            role="tablist"
            aria-label="Queryon modes"
            className="mt-7 inline-flex gap-1 rounded-full border bg-muted/60 p-1"
          >
            {HERO_MODES.map((mode, index) => {
              const isActive = index === activeIndex;
              const Icon = mode.icon;
              return (
                <button
                  key={mode.id}
                  type="button"
                  role="tab"
                  aria-selected={isActive}
                  onClick={() => {
                    setInteracted(true);
                    setActiveIndex(index);
                  }}
                  className="relative flex items-center gap-1.5 rounded-full px-4 py-1.5 text-sm font-medium"
                >
                  {isActive && (
                    <motion.span
                      layoutId="heroMode"
                      className="absolute inset-0 rounded-full bg-ink dark:bg-brand"
                      transition={{
                        type: "spring",
                        stiffness: 420,
                        damping: 34,
                      }}
                    />
                  )}
                  <Icon
                    className={cn(
                      "relative size-3.5 transition-colors",
                      isActive
                        ? "text-brand-bright dark:text-[#04140a]"
                        : "text-muted-foreground",
                    )}
                  />
                  <span
                    className={cn(
                      "relative transition-colors",
                      isActive
                        ? "text-white dark:text-[#04140a]"
                        : "text-muted-foreground",
                    )}
                  >
                    {mode.label}
                  </span>
                </button>
              );
            })}
          </div>

          <h1 className="mt-7 min-h-[2.3em] text-[clamp(40px,6vw,76px)] leading-[1.02] font-semibold tracking-[-0.045em]">
            <AnimatePresence mode="wait" initial={false}>
              <motion.span
                key={active.id}
                className="block"
                exit={{
                  opacity: 0,
                  y: -14,
                  filter: "blur(6px)",
                  transition: { duration: 0.22 },
                }}
              >
                <AnimatedLine text={active.title[0]} delay={0} />
                <span className="block text-brand">
                  <AnimatedLine text={active.title[1]} delay={0.12} />
                </span>
              </motion.span>
            </AnimatePresence>
          </h1>

          <p className="mt-6 max-w-xl text-[17px] leading-relaxed text-muted-foreground">
            A fast, native desktop client for Postgres, MySQL, SQL Server,
            ClickHouse and a dozen more — browse schemas, run queries
            and edit data without switching tools.
          </p>

          <div className="mt-8 flex flex-wrap items-center gap-3">
            <CtaButton href="/docs/install">Download for free</CtaButton>
            <CtaButton href="/examples" tone="light">
              Try it in the browser
            </CtaButton>
          </div>

          <ul className="mt-8 flex flex-wrap items-center gap-x-5 gap-y-2 text-sm text-muted-foreground">
            {["Windows", "macOS", "Linux"].map((os) => (
              <li key={os} className="flex items-center gap-2">
                <span className="size-1.5 rounded-full bg-brand" />
                {os}
              </li>
            ))}
            <li className="rounded-full border px-3 py-1 text-xs">
              No account required
            </li>
          </ul>
        </div>

        <motion.div
          initial={{ opacity: 0, scale: 0.94 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ duration: 1, ease: EASE, delay: 0.15 }}
          className="relative mx-auto aspect-square w-full max-w-[560px]"
        >
          <div className="console-lines absolute inset-0 overflow-hidden rounded-[36px] bg-ink shadow-[0_40px_120px_-40px_rgba(21,128,61,0.55)] ring-1 ring-white/5">
            <div
              aria-hidden
              className="absolute inset-0 bg-[radial-gradient(60%_55%_at_50%_45%,rgba(74,222,128,0.16),transparent_70%)]"
            />
            <HeroScene />
            <div className="absolute inset-x-5 top-5 flex items-center justify-between font-mono text-[11px] text-white/45">
              <span className="flex items-center gap-2">
                <span className="size-1.5 animate-pulse-dot rounded-full bg-brand-bright" />
                live · devdb
              </span>
              <span>3 schemas · 42 tables</span>
            </div>
          </div>
          {STATUS_CARDS.map((card, i) => (
            <motion.div
              key={card.title}
              initial={{ opacity: 0, y: 16 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.7, ease: EASE, delay: 0.8 + i * 0.18 }}
              className={cn("absolute", card.className)}
            >
              <div
                className={cn(
                  "flex animate-float items-center gap-3 rounded-2xl px-3.5 py-2.5 shadow-[0_20px_50px_-20px_rgba(0,0,0,0.45)] backdrop-blur-xl",
                  card.dark
                    ? "border border-brand-bright/30 bg-ink/85 text-white"
                    : "border bg-background/90",
                )}
                style={{ animationDelay: `${-i * 2.2}s` }}
              >
                <span
                  className={cn(
                    "grid size-8 place-items-center rounded-xl",
                    card.dark
                      ? "bg-brand-bright text-[#04140a]"
                      : "bg-ink text-brand-bright",
                  )}
                >
                  <card.icon className="size-4" />
                </span>
                <span>
                  <span className="block text-sm font-medium">
                    {card.title}
                  </span>
                  <span
                    className={cn(
                      "block font-mono text-[11px]",
                      card.dark ? "text-white/55" : "text-muted-foreground",
                    )}
                  >
                    {card.detail}
                  </span>
                </span>
              </div>
            </motion.div>
          ))}
        </motion.div>
      </div>

      <div
        ref={shotRef}
        className="mx-auto max-w-6xl px-5 pt-10 pb-6 sm:px-6 [perspective:1600px]"
      >
        <motion.div
          style={{ rotateX, scale, y: shotY, transformOrigin: "50% 0%" }}
          className="rounded-[22px] bg-gradient-to-b from-brand-bright/40 to-transparent p-px"
        >
          <div className="rounded-[21px] bg-background p-2 shadow-[0_50px_120px_-50px_rgba(5,46,22,0.6)]">
            <DataGridMock fixedHeight />
          </div>
        </motion.div>
      </div>
    </section>
  );
}
