import Link from "next/link"

const FOOTER_LINKS = [
  { href: "#features", label: "Features" },
  { href: "#engines", label: "Engines" },
  { href: "/examples", label: "Examples" },
  { href: "/docs", label: "Docs" },
  { href: "https://queryon.featurebase.app", label: "Feedback", external: true },
]

export function SiteFooter() {
  return (
    <footer className="border-t">
      <div className="mx-auto flex max-w-7xl flex-col items-center justify-between gap-6 px-6 py-12 text-sm text-muted-foreground sm:flex-row sm:gap-4">
        <span>© {new Date().getFullYear()} Queryon.</span>
        <div className="flex flex-wrap items-center justify-center gap-x-8 gap-y-3">
          {FOOTER_LINKS.map((link) =>
            link.external ? (
              <a
                key={link.href}
                href={link.href}
                className="transition-colors hover:text-foreground"
              >
                {link.label}
              </a>
            ) : (
              <Link
                key={link.href}
                href={link.href}
                className="transition-colors hover:text-foreground"
              >
                {link.label}
              </Link>
            )
          )}
        </div>
      </div>
    </footer>
  )
}
