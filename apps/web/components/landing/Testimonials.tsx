"use client";

import { motion } from "motion/react";

import { cn } from "@queryon/ui/lib/utils";

import { SectionLabel } from "./Brand";

interface Testimonial {
  quote: string;
  headline: string;
  name: string;
  title: string;
  tone?: "dark" | "brand";
}

const TESTIMONIALS: Testimonial[] = [
  {
    headline: "Finally, a database client that doesn't feel overwhelming.",
    quote:
      "I mostly work with Postgres, and Queryon has been much easier to navigate than the heavier database tools I've used before. The interface is clean, and I can get to the data I need without digging through five different panels.",
    name: "Arjun Mehta",
    title: "Full-stack Developer",
    tone: "dark",
  },
  {
    headline: "The table view is ridiculously convenient.",
    quote:
      "What I like most about Queryon is how quickly I can open a table, inspect the data, and start working with it. It feels more like a modern developer tool than an old-school database admin application.",
    name: "Rahul Sharma",
    title: "Backend Engineer",
  },
  {
    headline:
      "I stopped opening my browser every time I needed to check a database.",
    quote:
      "I wanted something lightweight for day-to-day database work, and Queryon fits that really well. It's fast, straightforward, and doesn't get in the way.",
    name: "Daniel Brooks",
    title: "Software Engineer",
  },
  {
    headline: "Much cleaner than the database clients I was using before.",
    quote:
      "The UI was what initially caught my attention, but the workflow is what made me keep using it. Connecting to a database and exploring tables feels surprisingly simple.",
    name: "Karan Patel",
    title: "Full-stack Engineer",
    tone: "brand",
  },
  {
    headline: "It feels like someone actually designed this for developers.",
    quote:
      "A lot of database tools feel like they were built around features first and usability second. Queryon feels different. The important stuff is easy to find, and the interface stays out of the way.",
    name: "Michael Chen",
    title: "Backend Developer",
  },
  {
    headline:
      "Great for quickly jumping into a database and figuring out what's going on.",
    quote:
      "I use database clients mostly for debugging and inspecting data. Queryon makes those quick tasks really easy without making me feel like I'm opening a full administration suite.",
    name: "Aman Verma",
    title: "Software Developer",
  },
];

function initials(name: string): string {
  return name
    .split(" ")
    .map((part) => part[0])
    .join("")
    .toUpperCase();
}

export function Testimonials() {
  return (
    <section className="mx-auto max-w-7xl px-5 sm:px-6">
      <div className="flex flex-col items-start justify-between gap-4 md:flex-row md:items-end">
        <div>
          <SectionLabel>Loved by developers</SectionLabel>
          <h2 className="mt-4 text-[clamp(28px,3.4vw,44px)] font-semibold tracking-[-0.035em]">
            What people <span className="text-brand">are saying.</span>
          </h2>
        </div>
        <p className="max-w-sm text-muted-foreground">
          Early feedback from developers using Queryon day to day.
        </p>
      </div>

      <div className="mt-10 columns-1 gap-3 sm:columns-2 lg:columns-3">
        {TESTIMONIALS.map((t, i) => (
          <motion.figure
            key={t.name}
            initial={{ opacity: 0, y: 24 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: "-8%" }}
            transition={{
              duration: 0.6,
              ease: [0.16, 1, 0.3, 1],
              delay: (i % 3) * 0.08,
            }}
            className={cn(
              "mb-3 flex break-inside-avoid flex-col gap-6 rounded-[24px] border p-6",
              t.tone === "dark" &&
                "console-lines border-transparent bg-ink text-white",
              t.tone === "brand" &&
                "border-transparent bg-brand-bright text-[#04140a]",
              !t.tone && "bg-card",
            )}
          >
            <span
              aria-hidden
              className={cn(
                "font-serif text-6xl leading-[0.5]",
                t.tone === "dark"
                  ? "text-brand-bright"
                  : t.tone === "brand"
                    ? "text-[#04140a]/50"
                    : "text-brand",
              )}
            >
              &ldquo;
            </span>
            <blockquote>
              <p
                className={cn(
                  "font-medium tracking-tight text-balance",
                  t.tone ? "text-xl" : "text-lg",
                )}
              >
                {t.headline}
              </p>
              <p
                className={cn(
                  "mt-3 text-sm leading-relaxed",
                  t.tone ? "opacity-70" : "text-muted-foreground",
                )}
              >
                {t.quote}
              </p>
            </blockquote>
            <figcaption className="flex items-center gap-3">
              <span
                className={cn(
                  "flex size-10 shrink-0 items-center justify-center rounded-full text-xs font-semibold",
                  t.tone === "dark"
                    ? "bg-brand-bright text-[#04140a]"
                    : t.tone === "brand"
                      ? "bg-ink text-brand-bright"
                      : "bg-brand-soft text-brand",
                )}
              >
                {initials(t.name)}
              </span>
              <div className="text-sm">
                <div className="font-medium">{t.name}</div>
                <div
                  className={t.tone ? "opacity-60" : "text-muted-foreground"}
                >
                  {t.title}
                </div>
              </div>
            </figcaption>
          </motion.figure>
        ))}
      </div>
    </section>
  );
}
