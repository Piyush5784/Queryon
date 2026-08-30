import Link from "next/link"
import { Database } from "lucide-react"

import { ThemeToggle } from "../../components/theme-toggle"

const NAV = [
  { href: "/docs", label: "Overview" },
  { href: "/docs/install", label: "Install" },
]

export default function DocsLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex min-h-svh flex-col">
      <header className="border-b">
        <div className="mx-auto flex max-w-6xl items-center justify-between px-6 py-4">
          <Link href="/" className="flex items-center gap-2 font-medium">
            <Database className="size-5" />
            Queryon
          </Link>
          <nav className="flex items-center gap-6 text-sm text-muted-foreground">
            <Link href="/#features" className="hover:text-foreground">
              Features
            </Link>
            <Link href="/docs" className="text-foreground">
              Docs
            </Link>
            <ThemeToggle />
          </nav>
        </div>
      </header>

      <div className="mx-auto flex w-full max-w-6xl flex-1 gap-10 px-6 py-10">
        <aside className="hidden w-48 shrink-0 sm:block">
          <nav className="sticky top-10 flex flex-col gap-1 text-sm">
            {NAV.map((item) => (
              <Link
                key={item.href}
                href={item.href}
                className="rounded-md px-3 py-1.5 text-muted-foreground hover:bg-muted hover:text-foreground"
              >
                {item.label}
              </Link>
            ))}
          </nav>
        </aside>
        <main className="min-w-0 flex-1">{children}</main>
      </div>
    </div>
  )
}
