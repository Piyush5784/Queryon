"use client"

import { useState } from "react"
import { motion, useScroll, useTransform, type MotionValue } from "motion/react"

function TextRevealWord({
  children,
  progress,
  range,
}: {
  children: string
  progress: MotionValue<number>
  range: [number, number]
}) {
  const opacity = useTransform(progress, range, [0, 1])
  return (
    <span className="relative mx-1 lg:mx-1.5">
      <span className="absolute text-muted-foreground/30">{children}</span>
      <motion.span style={{ opacity }} className="text-foreground">
        {children}
      </motion.span>
    </span>
  )
}

export function TextReveal({ children }: { children: string }) {
  const [targetRef, setTargetRef] = useState<HTMLDivElement | null>(null)
  const { scrollYProgress } = useScroll({
    target: { current: targetRef },
  })

  const words = children.split(" ")

  return (
    <div ref={setTargetRef} className="relative z-0 ">
      <div className="sticky top-0 mx-auto flex h-[50%] max-w-4xl items-center justify-center px-4 ">
        <span className="flex flex-wrap justify-center gap-y-4 p-5 text-center text-3xl font-medium text-balance md:gap-y-6 md:text-[44px]">
          {words.map((word, i) => {
            const start = i / words.length
            const end = start + 1 / words.length
            return (
              <TextRevealWord key={i} progress={scrollYProgress} range={[start, end]}>
                {word}
              </TextRevealWord>
            )
          })}
        </span>
      </div>
    </div>
  )
}
