import Link from "next/link";
import { ArrowUpRight } from "lucide-react";

import { cn } from "@queryon/ui/lib/utils";

export function LogoMark({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 32 32" className={className} aria-hidden="true">
      <rect width="32" height="32" rx="9" fill="#070a08" />
      <ellipse
        cx="16"
        cy="10"
        rx="8"
        ry="3"
        fill="none"
        stroke="#4ade80"
        strokeWidth="2"
      />
      <path
        d="M8 10v5c0 1.7 3.6 3 8 3s8-1.3 8-3v-5"
        fill="none"
        stroke="#4ade80"
        strokeWidth="2"
      />
      <path
        d="M8 15v5c0 1.7 3.6 3 8 3 1.4 0 2.7-.1 3.8-.4"
        fill="none"
        stroke="#4ade80"
        strokeWidth="2"
      />
      <path
        d="m21.5 20.5 3.5 3.5"
        stroke="#4ade80"
        strokeWidth="2.4"
        strokeLinecap="round"
      />
    </svg>
  );
}

export function Logo({ className }: { className?: string }) {
  return (
    <Link
      href="/"
      className={cn(
        "flex items-center gap-2 font-medium tracking-tight",
        className,
      )}
    >
      <LogoMark className="size-7" />
      <span className="text-[17px]">Queryon</span>
    </Link>
  );
}

interface CtaButtonProps {
  href: string;
  children: React.ReactNode;
  tone?: "dark" | "light" | "brand";
  external?: boolean;
  className?: string;
}

export function CtaButton({
  href,
  children,
  tone = "dark",
  external,
  className,
}: CtaButtonProps) {
  const styles = {
    dark: "bg-ink text-white hover:bg-ink-2 dark:bg-brand dark:text-[#04140a] dark:hover:bg-brand/90",
    light: "border bg-background text-foreground hover:bg-muted",
    brand: "bg-brand-bright text-[#04140a] hover:bg-brand-bright/90",
  }[tone];
  const chip = {
    dark: "bg-brand-bright text-[#04140a] dark:bg-ink dark:text-brand-bright",
    light: "bg-ink text-brand-bright dark:bg-brand dark:text-[#04140a]",
    brand: "bg-ink text-brand-bright",
  }[tone];
  const classes = cn(
    "group inline-flex h-12 items-center gap-3 rounded-full py-1.5 pr-1.5 pl-6 text-[15px] font-medium transition-colors",
    styles,
    className,
  );
  const content = (
    <>
      {children}
      <span
        className={cn(
          "grid size-9 place-items-center rounded-full transition-transform duration-300 group-hover:rotate-45",
          chip,
        )}
      >
        <ArrowUpRight className="size-4" />
      </span>
    </>
  );
  return external ? (
    <a
      href={href}
      target="_blank"
      rel="noopener noreferrer"
      className={classes}
    >
      {content}
    </a>
  ) : (
    <Link href={href} className={classes}>
      {content}
    </Link>
  );
}

export function SectionLabel({
  children,
  className,
}: {
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <p
      className={cn(
        "flex items-center gap-2 font-mono text-xs tracking-[0.18em] text-muted-foreground uppercase",
        className,
      )}
    >
      <span className="size-1.5 animate-pulse-dot rounded-full bg-brand-bright" />
      {children}
    </p>
  );
}
