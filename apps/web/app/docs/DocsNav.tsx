"use client"

import Link from "next/link"
import { usePathname } from "next/navigation"
import { BookOpen, Download } from "lucide-react"

import { cn } from "@queryon/ui/lib/utils"

const NAV = [
  { href: "/docs", label: "Overview", icon: BookOpen },
  { href: "/docs/install", label: "Install", icon: Download },
]

export function DocsNav() {
  const pathname = usePathname()
  return (
    <nav className="flex gap-1 text-sm sm:sticky sm:top-28 sm:flex-col">
      <p className="mb-3 hidden px-3 font-mono text-[11px] tracking-[0.16em] text-muted-foreground uppercase sm:block">Getting started</p>
      {NAV.map(({ href, label, icon: Icon }) => {
        const active = pathname === href
        return (
          <Link
            key={href}
            href={href}
            aria-current={active ? "page" : undefined}
            className={cn(
              "flex items-center gap-2.5 rounded-full px-3 py-2 transition-colors",
              active ? "bg-brand-soft font-medium text-brand" : "text-muted-foreground hover:bg-muted hover:text-foreground"
            )}
          >
            <Icon className="size-4" />
            {label}
          </Link>
        )
      })}
    </nav>
  )
}
