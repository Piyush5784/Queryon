import { SectionLabel } from "./Brand"
import { ENGINE_LOGOS } from "./engineLogos"

function LogoItem({ name, Logo }: { name: string; Logo: () => React.JSX.Element }) {
  return (
    <div className="flex shrink-0 items-center gap-3 rounded-2xl border bg-card px-5 py-3 transition-colors hover:border-brand/50">
      <div className="size-8 shrink-0">
        <Logo />
      </div>
      <span className="text-[15px] font-medium whitespace-nowrap">{name}</span>
    </div>
  )
}

export function EngineMarquee() {
  const rows = [ENGINE_LOGOS, [...ENGINE_LOGOS].reverse()]
  return (
    <section id="engines" className="scroll-mt-24 py-20">
      <div className="mx-auto mb-10 flex max-w-7xl flex-col items-start justify-between gap-4 px-5 sm:px-6 md:flex-row md:items-end">
        <div>
          <SectionLabel>Supported databases</SectionLabel>
          <h2 className="mt-4 text-[clamp(28px,3.4vw,44px)] font-semibold tracking-[-0.035em]">
            One window. <span className="text-brand">Every engine.</span>
          </h2>
        </div>
        <p className="max-w-sm text-muted-foreground">
          Every engine gets its own real driver — so data types, schema editing and constraints behave the way that engine
          actually does.
        </p>
      </div>
      <div className="flex flex-col gap-3" aria-label="Supported database engines">
        {rows.map((row, r) => (
          <div key={r} className="group flex overflow-hidden [mask-image:linear-gradient(to_right,transparent,black_8%,black_92%,transparent)]">
            <div
              className="flex w-max gap-3 pr-3 group-hover:[animation-play-state:paused]"
              style={{ animation: `marquee-rtl ${r ? 44 : 36}s linear infinite`, animationDirection: r ? "reverse" : "normal" }}
            >
              {[...row, ...row].map(({ name, Logo }, i) => (
                <LogoItem key={`${name}-${i}`} name={name} Logo={Logo} />
              ))}
            </div>
          </div>
        ))}
      </div>
    </section>
  )
}
