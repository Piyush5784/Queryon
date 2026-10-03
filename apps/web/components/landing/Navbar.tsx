"use client"

import { useEffect, useState } from "react"
import Link from "next/link"
import { usePathname } from "next/navigation"
import { Code2, Menu, X } from "lucide-react"
import { AnimatePresence, motion } from "motion/react"

import { buttonVariants } from "@queryon/ui/components/button"
import { cn } from "@queryon/ui/lib/utils"

import { ThemeToggle } from "../theme-toggle"
import { CtaButton, Logo } from "./Brand"

const NAV_LINKS = [
  { href: "/#features", label: "Features" },
  { href: "/#engines", label: "Engines" },
  { href: "/examples", label: "Examples" },
  { href: "/docs", label: "Docs" },
  { href: "https://queryon.featurebase.app", label: "Feedback", external: true },
]

const GITHUB_URL = "https://github.com/Piyush5784/Queryon"

export function Navbar() {
  const [showMenu, setShowMenu] = useState(false)
  const [scrolled, setScrolled] = useState(false)
  const pathname = usePathname()
  const [menuPath, setMenuPath] = useState(pathname)
  if (menuPath !== pathname) {
    setMenuPath(pathname)
    setShowMenu(false)
  }

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 16)
    onScroll()
    window.addEventListener("scroll", onScroll, { passive: true })
    return () => window.removeEventListener("scroll", onScroll)
  }, [])

  return (
    <motion.header
      initial={{ y: -40, opacity: 0 }}
      animate={{ y: 0, opacity: 1 }}
      transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
      className="sticky top-0 z-50 px-3 pt-3 sm:px-4"
    >
      <div
        className={cn(
          "mx-auto flex h-14 max-w-7xl items-center justify-between rounded-full border pr-2 pl-4 transition-[background-color,box-shadow,border-color] duration-500",
          scrolled
            ? "border-border bg-background/80 shadow-[0_10px_40px_-20px_rgba(0,0,0,0.25)] backdrop-blur-xl"
            : "border-transparent bg-transparent"
        )}
      >
        <Logo />

        <nav aria-label="Primary" className="hidden items-center gap-1 md:flex">
          {NAV_LINKS.map((link) => {
            const active = !link.external && pathname === link.href
            const className = cn(
              "rounded-full px-3.5 py-2 text-sm transition-colors",
              active ? "bg-brand-soft text-foreground" : "text-muted-foreground hover:text-foreground"
            )
            return link.external ? (
              <a key={link.href} href={link.href} className={className}>
                {link.label}
              </a>
            ) : (
              <Link key={link.href} href={link.href} className={className}>
                {link.label}
              </Link>
            )
          })}
        </nav>

        <div className="flex items-center gap-2">
          <a
            href={GITHUB_URL}
            target="_blank"
            rel="noopener noreferrer"
            aria-label="View source on GitHub"
            className={cn(buttonVariants({ size: "icon-sm", variant: "outline" }), "hidden rounded-full sm:inline-flex")}
          >
            <Code2 className="size-4" />
          </a>
          <ThemeToggle className="hidden rounded-full sm:inline-flex" />
          <CtaButton href="/docs/install" className="hidden h-10 gap-2 pl-4 text-sm sm:inline-flex [&>span]:size-7">
            Download
          </CtaButton>
          <button
            type="button"
            aria-label={showMenu ? "Close menu" : "Open menu"}
            aria-expanded={showMenu}
            onClick={() => setShowMenu((v) => !v)}
            className={cn(buttonVariants({ size: "icon-sm", variant: "outline" }), "rounded-full md:hidden")}
          >
            {showMenu ? <X className="size-4" /> : <Menu className="size-4" />}
          </button>
        </div>
      </div>

      <AnimatePresence>
        {showMenu && (
          <motion.div
            initial={{ opacity: 0, y: -8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -8 }}
            transition={{ duration: 0.25 }}
            className="absolute inset-x-3 mt-2 md:hidden"
          >
            <div className="flex flex-col gap-1 rounded-3xl border bg-background/95 p-2 shadow-xl backdrop-blur-xl">
              {[...NAV_LINKS, { href: GITHUB_URL, label: "GitHub", external: true }].map((link) => {
                const className =
                  "rounded-2xl px-4 py-3 text-[15px] text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
                return link.external ? (
                  <a key={link.href} href={link.href} className={className}>
                    {link.label}
                  </a>
                ) : (
                  <Link key={link.href} href={link.href} className={className}>
                    {link.label}
                  </Link>
                )
              })}
              <div className="mt-1 flex items-center gap-2 p-1">
                <CtaButton href="/docs/install" className="flex-1 justify-between">
                  Download
                </CtaButton>
                <ThemeToggle className="size-12 rounded-full" />
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.header>
  )
}
