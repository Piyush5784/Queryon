"use client";

import { useRef } from "react";
import {
  motion,
  useScroll,
  useTransform,
  type MotionValue,
} from "motion/react";

const HIGHLIGHT = new Set(["whole", "day", "inside", "database."]);

function TextRevealWord({
  children,
  progress,
  range,
}: {
  children: string;
  progress: MotionValue<number>;
  range: [number, number];
}) {
  const opacity = useTransform(progress, range, [0, 1]);
  const y = useTransform(progress, range, [8, 0]);
  const lit = HIGHLIGHT.has(children);
  return (
    <span className="relative mx-[0.12em]">
      <span className="absolute text-foreground/10">{children}</span>
      <motion.span
        style={{ opacity, y }}
        className={
          lit ? "inline-block text-brand" : "inline-block"
        }
      >
        {children}
      </motion.span>
    </span>
  );
}

export function TextReveal({ children }: { children: string }) {
  const ref = useRef<HTMLDivElement>(null);
  const { scrollYProgress } = useScroll({
    target: ref,
    offset: ["start start", "end end"],
  });
  const words = children.split(" ");

  return (
    <div ref={ref} className="relative h-[220vh]">
      <div className="sticky top-0 flex h-screen items-center justify-center px-5">
        <div
          aria-hidden
          className="grid-lines absolute inset-0 -z-10 [mask-image:radial-gradient(50%_50%_at_50%_50%,black,transparent)]"
        />
        <p className="flex max-w-5xl flex-wrap justify-center gap-y-2 text-center text-[clamp(30px,4.6vw,60px)] leading-[1.1] font-semibold tracking-[-0.035em] text-balance">
          {words.map((word, i) => {
            const start = (i / words.length) * 0.85;
            const end = start + 0.85 / words.length;
            return (
              <TextRevealWord
                key={i}
                progress={scrollYProgress}
                range={[start, end]}
              >
                {word}
              </TextRevealWord>
            );
          })}
        </p>
      </div>
    </div>
  );
}
