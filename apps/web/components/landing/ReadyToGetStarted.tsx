import Link from "next/link"
import { ArrowRight } from "lucide-react"

import { buttonVariants } from "@queryon/ui/components/button"
import { cn } from "@queryon/ui/lib/utils"

export function ReadyToGetStarted() {
  return (
    <div className="relative mx-auto flex min-h-[300px] pt-0  mb-10 mt-0 w-[calc(100%-20px)] max-w-[1000px] flex-col items-center justify-center overflow-hidden rounded-[20px] border bg-card p-8 text-center">
      <div
        aria-hidden
        className="pointer-events-none absolute inset-x-0 -z-10 mx-auto h-[300px] max-w-[1000px] [mask-image:radial-gradient(closest-side,black,transparent)]"
      >
        <div className="absolute inset-0 bg-[linear-gradient(to_right,var(--border)_1px,transparent_1px),linear-gradient(to_bottom,var(--border)_1px,transparent_1px)] bg-[size:32px_32px] opacity-40" />
      </div>
      <h2 className="mb-6 max-w-xl text-3xl font-medium tracking-tight text-balance sm:text-4xl">
        Get started in under a minute
      </h2>
      <div className="flex flex-col items-center justify-center gap-3 sm:flex-row">
        <Link href="/docs/install" className={cn(buttonVariants({ size: "lg" }), "gap-2")}>
          Download Queryon
          <ArrowRight className="size-4" />
        </Link>
      </div>
      <p className="mt-4 text-sm text-muted-foreground">
        Windows, macOS, and Linux · No account required
      </p>
    </div>
  )
}
