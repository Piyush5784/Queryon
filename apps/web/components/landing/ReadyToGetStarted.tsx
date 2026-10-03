"use client"

import { motion } from "motion/react"

import { CtaButton, LogoMark } from "./Brand"

const TERMINAL = [
  { prompt: "$", text: "queryon --connect postgres://devdb" },
  { prompt: "✓", text: "connected in 184 ms · keychain", brand: true },
  { prompt: ">", text: "SELECT count(*) FROM users;" },
  { prompt: "✓", text: "1 row · 4 ms", brand: true },
]

export function ReadyToGetStarted() {
  return (
    <section className="px-3 pb-10 sm:px-4">
      <div className="console-lines relative mx-auto grid max-w-7xl items-center gap-12 overflow-hidden rounded-[36px] bg-ink px-6 py-16 text-white sm:px-14 lg:grid-cols-[1.1fr_1fr] lg:py-20">
        <div aria-hidden className="absolute -bottom-40 -left-20 h-96 w-[60%] rounded-full bg-brand-bright/25 blur-[120px]" />
        <div className="relative">
          <LogoMark className="size-12 ring-1 ring-white/10 rounded-[14px]" />
          <h2 className="mt-8 text-[clamp(34px,4.6vw,60px)] leading-[1.02] font-semibold tracking-[-0.04em]">
            Get started in <span className="font-serif font-normal text-brand-bright italic">under a minute.</span>
          </h2>
          <p className="mt-5 max-w-md text-white/60">Download, connect, query. No account, no license key, no telemetry wall.</p>
          <div className="mt-8 flex flex-wrap gap-3">
            <CtaButton href="/docs/install" tone="brand">
              Download Queryon
            </CtaButton>
            <CtaButton href="/docs" tone="light">
              Read the docs
            </CtaButton>
          </div>
          <p className="mt-6 font-mono text-xs text-white/40">Windows · macOS · Linux · No account required</p>
        </div>

        <div className="relative rounded-2xl border border-white/10 bg-black/40 p-5 font-mono text-[13px] backdrop-blur">
          <div className="mb-4 flex gap-1.5">
            <span className="size-2.5 rounded-full bg-white/15" />
            <span className="size-2.5 rounded-full bg-white/15" />
            <span className="size-2.5 rounded-full bg-brand-bright" />
          </div>
          {TERMINAL.map((line, i) => (
            <motion.div
              key={line.text}
              initial={{ opacity: 0, x: -10 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
              transition={{ delay: 0.25 + i * 0.35, duration: 0.4 }}
              className="flex gap-3 py-1"
            >
              <span className={line.brand ? "text-brand-bright" : "text-white/35"}>{line.prompt}</span>
              <span className={line.brand ? "text-brand-bright/80" : "text-white/85"}>{line.text}</span>
            </motion.div>
          ))}
          <div className="flex gap-3 py-1">
            <span className="text-white/35">&gt;</span>
            <span className="h-4 w-2 animate-pulse bg-brand-bright" />
          </div>
        </div>
      </div>
    </section>
  )
}
