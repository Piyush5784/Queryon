import { ENGINE_LOGOS } from "./engineLogos"

function LogoItem({ name, Logo }: { name: string; Logo: () => React.JSX.Element }) {
  return (
    <div className="flex shrink-0 items-center gap-3 px-10">
      <div className="size-12 shrink-0">
        <Logo />
      </div>
      <span className="text-base font-medium whitespace-nowrap text-foreground">{name}</span>
    </div>
  )
}

export function EngineMarquee() {
  return (
    <section id="engines" className="py-16">
      <p className="mb-6 text-2xl text-center  font-medium text-muted-foreground">
        Supported databases
      </p>
      <div
        className="group flex overflow-hidden [mask-image:linear-gradient(to_right,transparent,black_8%,black_92%,transparent)]"
        aria-label="Supported database engines"
      >
        <div className="flex w-max animate-[marquee-rtl_32s_linear_infinite] group-hover:[animation-play-state:paused]">
          {[...ENGINE_LOGOS, ...ENGINE_LOGOS].map(({ name, Logo }, i) => (
            <LogoItem key={`${name}-${i}`} name={name} Logo={Logo} />
          ))}
        </div>
      </div>
    </section>
  )
}
