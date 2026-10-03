"use client";

import { useEffect, useRef } from "react";
import { ReactLenis, type LenisRef } from "lenis/react";
import { cancelFrame, frame, type FrameData } from "motion/react";

import "lenis/dist/lenis.css";

export function SmoothScroll({ children }: { children: React.ReactNode }) {
  const lenisRef = useRef<LenisRef>(null);

  useEffect(() => {
    const update = (data: FrameData) =>
      lenisRef.current?.lenis?.raf(data.timestamp);
    frame.update(update, true);
    return () => cancelFrame(update);
  }, []);

  return (
    <ReactLenis
      root
      ref={lenisRef}
      options={{ autoRaf: false, lerp: 0.1, anchors: { offset: -80 } }}
    >
      {children}
    </ReactLenis>
  );
}
