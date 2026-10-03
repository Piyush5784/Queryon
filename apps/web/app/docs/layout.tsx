import { Navbar } from "../../components/landing/Navbar";
import { SiteFooter } from "../../components/landing/SiteFooter";
import { DocsNav } from "./DocsNav";

export default function DocsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="flex min-h-svh flex-col">
      <Navbar />
      <div className="relative">
        <div
          aria-hidden
          className="grid-lines pointer-events-none absolute inset-x-0 top-0 -z-10 h-80 [mask-image:linear-gradient(to_bottom,black,transparent)]"
        />
        <div className="mx-auto flex w-full max-w-6xl flex-1 flex-col gap-8 px-5 py-12 sm:flex-row sm:gap-12 sm:px-6">
          <aside className="shrink-0 sm:w-52">
            <DocsNav />
          </aside>
          <main className="min-w-0 flex-1 pb-16">{children}</main>
        </div>
      </div>
      <SiteFooter />
    </div>
  );
}
