"use client"

import { useState } from "react"
import Link from "next/link"
import { Database, Menu, X } from "lucide-react"
import { motion } from "motion/react"

import { buttonVariants } from "@queryon/ui/components/button"
import {
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  navigationMenuTriggerStyle,
} from "@queryon/ui/components/navigation-menu"
import { cn } from "@queryon/ui/lib/utils"

const NAV_LINKS = [
  { href: "#features", label: "Features" },
  { href: "#engines", label: "Engines" },
  { href: "/examples", label: "Examples" },
  { href: "/docs", label: "Docs" },
  { href: "https://queryon.featurebase.app", label: "Feedback", external: true },
  { href: "https://github.com/Piyush5784/Queryon", label: "GitHub", external: true },
]

export function Navbar() {
  const [showMenu, setShowMenu] = useState(false)

  return (
    <motion.div
      initial={{ y: "-100px", opacity: 0 }}
      animate={{ y: "0px", opacity: 1 }}
      transition={{ duration: 0.5, ease: "easeOut" }}
      className="sticky top-0 z-50 px-4 pt-4 relative"
    >
      <div className="mx-auto flex max-w-7xl items-center justify-between rounded-lg border bg-background/10 bg-opacity-10 p-3 pl-5 shadow-sm backdrop-blur-sm">
        <Link href="/" className="flex items-center gap-2 font-medium">
          <div className="flex size-6 items-center justify-center rounded-md bg-foreground text-background">
            <Database className="size-3.5" />
          </div>
          Queryon
        </Link>

        <NavigationMenu className="hidden sm:flex ">
          <NavigationMenuList>
            {NAV_LINKS.map((link) => (
              <NavigationMenuItem key={link.href}>
                <NavigationMenuLink
                  render={link.external ? <a href={link.href} /> : <Link href={link.href} />}
                  className={cn(
                    navigationMenuTriggerStyle(),
                    "rounded-full text-muted-foreground hover:text-foreground"
                  )}
                >
                  {link.label}
                </NavigationMenuLink>
              </NavigationMenuItem>
            ))}
          </NavigationMenuList>
        </NavigationMenu>

        <div className="flex items-center gap-2">
          <Link
            href="/docs/install"
            className={cn(buttonVariants({ size: "sm" }), "hidden sm:inline-flex")}
          >
            Download
          </Link>
          <button
            type="button"
            aria-label={showMenu ? "Close menu" : "Open menu"}
            onClick={() => setShowMenu((v) => !v)}
            className={cn(
              buttonVariants({ size: "icon-sm", variant: "outline" }),
              "rounded-full sm:hidden"
            )}
          >
            {showMenu ? <X className="size-4" /> : <Menu className="size-4" />}
          </button>
        </div>
      </div>

      <motion.div
        initial={false}
        animate={showMenu ? { height: "auto", opacity: 1 } : { height: 0, opacity: 0 }}
        transition={{ duration: 0.3, ease: "easeOut" }}
        className="absolute inset-x-4 max-w-6xl overflow-hidden sm:hidden"
      >
        <div className="mt-2 flex flex-col items-stretch gap-1 rounded-3xl border bg-background/95 p-2 shadow-sm backdrop-blur-sm">
          {NAV_LINKS.map((link) => {
            const className =
              "rounded-full px-4 py-2 text-center text-sm text-muted-foreground transition-colors hover:bg-muted/50 hover:text-foreground"
            return link.external ? (
              <a
                key={link.href}
                href={link.href}
                onClick={() => setShowMenu(false)}
                className={className}
              >
                {link.label}
              </a>
            ) : (
              <Link
                key={link.href}
                href={link.href}
                onClick={() => setShowMenu(false)}
                className={className}
              >
                {link.label}
              </Link>
            )
          })}
          <Link
            href="/docs/install"
            onClick={() => setShowMenu(false)}
            className={cn(buttonVariants({ size: "sm" }), "mt-1 rounded-full")}
          >
            Download
          </Link>
        </div>
      </motion.div>
    </motion.div>
  )
}
