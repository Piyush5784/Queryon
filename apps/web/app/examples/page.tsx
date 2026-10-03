import { ExamplesWorkspaceClient } from "../../components/examples/ExamplesWorkspaceClient"
import { SectionLabel } from "../../components/landing/Brand"
import { Navbar } from "../../components/landing/Navbar"
import { SiteFooter } from "../../components/landing/SiteFooter"

export const metadata = {
  title: "Examples — Queryon",
  description: "See Queryon browse large tables and run queries, right in your browser.",
}

export default function ExamplesPage() {
  return (
    <div className="flex min-h-svh flex-col">
      <Navbar />
      <main className="flex-1">
        <div className="relative mx-auto flex max-w-6xl flex-col items-center px-5 pt-16 pb-4 text-center sm:px-6">
          <div aria-hidden className="grid-lines pointer-events-none absolute inset-0 -z-10 [mask-image:radial-gradient(60%_70%_at_50%_30%,black,transparent)]" />
          <SectionLabel>Live demo · runs in your browser</SectionLabel>
          <h1 className="mt-5 text-[clamp(34px,5vw,64px)] leading-[1.03] font-semibold tracking-[-0.045em] text-balance">
            Browse 10,000 rows <span className="text-brand">without breaking a sweat.</span>
          </h1>
          <p className="mx-auto mt-4 max-w-xl text-[17px] text-muted-foreground text-balance">
            This is the same grid that ships in the desktop app — canvas-rendered, virtualized, and
            fast at any scale. Click a table in the sidebar, or open the query tab and hit Run.
          </p>
        </div>

        <div className="mx-auto max-w-6xl px-5 pt-8 pb-24 sm:px-6">
          <div className="rounded-[24px] bg-gradient-to-b from-brand-bright/40 to-transparent p-px shadow-[0_50px_120px_-50px_rgba(5,46,22,0.5)]">
            <div className="rounded-[23px] bg-background p-2">
              <ExamplesWorkspaceClient />
            </div>
          </div>
        </div>
      </main>
      <SiteFooter />
    </div>
  )
}
