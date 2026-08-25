import Link from "next/link"

export function SiteFooter() {
  return (
    <footer className="border-t">
      <div className="mx-auto flex max-w-6xl flex-col items-center justify-between gap-4 px-6 py-8 text-sm text-muted-foreground sm:flex-row">
        <span>© {new Date().getFullYear()} Queryon.</span>
        <div className="flex items-center gap-6">
          <Link href="/docs" className="transition-colors hover:text-foreground">
            Docs
          </Link>
          <a
            href="https://github.com/Piyush5784/Queryon"
            className="transition-colors hover:text-foreground"
          >
            GitHub
          </a>
        </div>
      </div>
    </footer>
  )
}
