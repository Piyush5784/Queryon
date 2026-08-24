import type { MDXComponents } from "mdx/types"
import Link from "next/link"

import { cn } from "@queryon/ui/lib/utils"

export function useMDXComponents(components: MDXComponents): MDXComponents {
  return {
    h1: ({ className, ...props }) => (
      <h1 className={cn("mt-0 mb-4 text-3xl font-medium tracking-tight", className)} {...props} />
    ),
    h2: ({ className, ...props }) => (
      <h2 className={cn("mt-10 mb-3 text-xl font-medium tracking-tight", className)} {...props} />
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
            className={cn("font-medium text-foreground underline underline-offset-4", className)}
            {...props}
          />
        )
      }
      return (
        <a
          href={href}
          className={cn("font-medium text-foreground underline underline-offset-4", className)}
          {...props}
        />
      )
    },
    ul: ({ className, ...props }) => (
      <ul className={cn("mb-4 ml-6 list-disc text-muted-foreground", className)} {...props} />
    ),
    ol: ({ className, ...props }) => (
      <ol className={cn("mb-4 ml-6 list-decimal text-muted-foreground", className)} {...props} />
    ),
    li: ({ className, ...props }) => <li className={cn("mb-1", className)} {...props} />,
    code: ({ className, ...props }) => (
      <code
        className={cn("rounded bg-muted px-1.5 py-0.5 font-mono text-[0.85em]", className)}
        {...props}
      />
    ),
    pre: ({ className, ...props }) => (
      <pre
        className={cn("mb-4 overflow-x-auto rounded-lg bg-muted px-4 py-3 text-sm", className)}
        {...props}
      />
    ),
    hr: ({ className, ...props }) => <hr className={cn("my-8 border-border", className)} {...props} />,
    ...components,
  }
}
