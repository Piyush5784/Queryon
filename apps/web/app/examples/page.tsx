import { ExamplesWorkspaceClient } from "../../components/examples/ExamplesWorkspaceClient"
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
        <div className="mx-auto max-w-6xl px-6 pt-12 pb-4 text-center">
          <h1 className="text-3xl font-medium tracking-tight text-balance sm:text-4xl">
            Browse 10,000 rows without breaking a sweat
          </h1>
          <p className="mx-auto mt-3 max-w-xl text-muted-foreground text-balance">
            This is the same grid that ships in the desktop app — canvas-rendered, virtualized, and
            fast at any scale. Click a table in the sidebar, or open the query tab and hit Run.
          </p>
        </div>

        <div className="mx-auto max-w-6xl px-6 pt-6 pb-24">
          <ExamplesWorkspaceClient />
        </div>
      </main>
      <SiteFooter />
    </div>
  )
}
