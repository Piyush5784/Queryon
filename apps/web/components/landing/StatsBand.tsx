"use client"

import { useEffect, useRef } from "react"
import { animate, useInView } from "motion/react"

const STATS = [
  { value: 0.2, prefix: "~", suffix: "s", decimals: 1, label: "cold start, debug build" },
  { value: 12, suffix: "+", decimals: 0, label: "database engines" },
  { value: 0, decimals: 0, label: "accounts or sign-ups needed" },
  { value: 100, suffix: "%", decimals: 0, label: "credentials kept on your machine" },
]

function Count({ value, decimals, prefix = "", suffix = "" }: { value: number; decimals: number; prefix?: string; suffix?: string }) {
  const ref = useRef<HTMLSpanElement>(null)
  const inView = useInView(ref, { once: true, margin: "-15%" })
  useEffect(() => {
    const el = ref.current
    if (!el || !inView || window.matchMedia("(prefers-reduced-motion: reduce)").matches) return
    const controls = animate(0, value, {
      duration: 1.4,
      ease: [0.16, 1, 0.3, 1],
      onUpdate: (v) => {
        el.textContent = `${prefix}${v.toFixed(decimals)}${suffix}`
      },
    })
    return () => controls.stop()
  }, [inView, value, decimals, prefix, suffix])
  return (
    <span ref={ref} className="tabular-nums">
      {`${prefix}${value.toFixed(decimals)}${suffix}`}
    </span>
  )
}

export function StatsBand() {
  return (
    <section className="px-3 sm:px-4">
      <div className="console-lines relative mx-auto max-w-7xl overflow-hidden rounded-[32px] bg-ink px-6 py-14 text-white sm:px-12">
        <div aria-hidden className="absolute -top-24 left-1/2 h-64 w-[70%] -translate-x-1/2 rounded-full bg-brand-bright/20 blur-[100px]" />
        <dl className="relative grid grid-cols-2 gap-x-6 gap-y-10 lg:grid-cols-4">
          {STATS.map((s) => (
            <div key={s.label} className="flex flex-col-reverse border-l border-white/10 pl-5">
              <dt className="mt-2 text-sm text-white/55">{s.label}</dt>
              <dd className="text-[clamp(40px,5vw,64px)] leading-none font-semibold tracking-[-0.04em] text-brand-bright">
                <Count value={s.value} decimals={s.decimals} prefix={s.prefix} suffix={s.suffix} />
              </dd>
            </div>
          ))}
        </dl>
      </div>
    </section>
  )
}
