import Link from "next/link";

import { Logo } from "./Brand";

const COLUMNS = [
  {
    title: "Product",
    links: [
      { href: "/#features", label: "Features" },
      { href: "/#engines", label: "Engines" },
      { href: "/examples", label: "Examples" },
    ],
  },
  {
    title: "Resources",
    links: [
      { href: "/docs", label: "Docs" },
      { href: "/docs/install", label: "Install" },
    ],
  },
  {
    title: "Community",
    links: [
      {
        href: "https://queryon.featurebase.app",
        label: "Feedback",
        external: true,
      },
      {
        href: "https://github.com/Piyush5784/Queryon",
        label: "GitHub",
        external: true,
      },
    ],
  },
];

export function SiteFooter() {
  return (
    <footer className="relative overflow-hidden border-t">
      <div className="mx-auto grid max-w-7xl gap-12 px-5 pt-16 pb-10 sm:px-6 md:grid-cols-[1.4fr_repeat(3,1fr)]">
        <div>
          <Logo />
          <p className="mt-4 max-w-xs text-sm text-muted-foreground">
            A fast, native desktop client for every database you work with.
          </p>
          <span className="mt-6 inline-flex items-center gap-2 rounded-full border px-3 py-1 font-mono text-[11px] text-muted-foreground">
            <span className="size-1.5 animate-pulse-dot rounded-full bg-brand" />
            All systems local
          </span>
        </div>
        {COLUMNS.map((col) => (
          <div key={col.title}>
            <h3 className="font-mono text-[11px] tracking-[0.16em] text-muted-foreground uppercase">
              {col.title}
            </h3>
            <ul className="mt-4 flex flex-col gap-2.5 text-sm">
              {col.links.map((link) => (
                <li key={link.href}>
                  {"external" in link ? (
                    <a
                      href={link.href}
                      target="_blank"
                      rel="noreferrer"
                      className="transition-colors hover:text-brand"
                    >
                      {link.label}
                    </a>
                  ) : (
                    <Link
                      href={link.href}
                      className="transition-colors hover:text-brand"
                    >
                      {link.label}
                    </Link>
                  )}
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>
      <div className="mx-auto flex max-w-7xl items-center justify-between border-t px-5 py-6 text-xs text-muted-foreground sm:px-6">
        <span>© {new Date().getFullYear()} Queryon.</span>
        <span className="font-mono">
          Built for people who live in databases.
        </span>
      </div>
      <div
        aria-hidden
        className="pointer-events-none -mb-[0.22em] text-center text-[22vw] leading-none font-semibold tracking-[-0.06em] text-foreground/[0.04] select-none"
      >
        Queryon
      </div>
    </footer>
  );
}
