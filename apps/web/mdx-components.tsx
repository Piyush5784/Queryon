import type { MDXComponents } from "mdx/types"
import Link from "next/link"

import { cn } from "@queryon/ui/lib/utils"

export function useMDXComponents(components: MDXComponents): MDXComponents {
  return {
    h1: ({ className, ...props }) => (
      <h1 className={cn("mt-0 mb-5 text-[clamp(32px,4vw,48px)] leading-[1.05] font-semibold tracking-[-0.04em]", className)} {...props} />
    ),
    h2: ({ className, ...props }) => (
      <h2 className={cn("mt-12 mb-3 flex items-center gap-2.5 text-2xl font-semibold tracking-[-0.02em] before:size-2 before:rounded-full before:bg-brand-bright before:content-['']", className)} {...props} />
    ),
    h3: ({ className, ...props }) => (
      <h3 className={cn("mt-8 mb-2 text-base font-medium", className)} {...props} />
    ),
    p: ({ className, ...props }) => (
      <p className={cn("mb-4 leading-relaxed text-muted-foreground", className)} {...props} />
    ),
    a: ({ className, href, ...props }) => {
      if (href?.startsWith("/")) {
        return (
          <Link
            href={href}
            className={cn("font-medium text-brand underline decoration-brand/30 underline-offset-4 transition-colors hover:decoration-brand", className)}
            {...props}
          />
        )
      }
      return (
        <a
          href={href}
          className={cn("font-medium text-brand underline decoration-brand/30 underline-offset-4 transition-colors hover:decoration-brand", className)}
          {...props}
        />
      )
    },
    ul: ({ className, ...props }) => (
      <ul className={cn("mb-4 ml-6 list-disc text-muted-foreground marker:text-brand", className)} {...props} />
    ),
    ol: ({ className, ...props }) => (
      <ol className={cn("mb-4 ml-6 list-decimal text-muted-foreground marker:font-mono marker:text-brand", className)} {...props} />
    ),
    li: ({ className, ...props }) => <li className={cn("mb-1", className)} {...props} />,
    code: ({ className, ...props }) => (
      <code
        className={cn("rounded-md bg-brand-soft px-1.5 py-0.5 font-mono text-[0.85em] text-brand", className)}
        {...props}
      />
    ),
    pre: ({ className, ...props }) => (
      <pre
        className={cn("console-lines mb-6 overflow-x-auto rounded-2xl bg-ink px-5 py-4 text-sm text-white [&_code]:bg-transparent [&_code]:p-0 [&_code]:text-brand-bright", className)}
        {...props}
      />
    ),
    hr: ({ className, ...props }) => <hr className={cn("my-8 border-border", className)} {...props} />,
    ...components,
  }
}
