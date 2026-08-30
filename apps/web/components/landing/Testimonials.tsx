interface Testimonial {
  quote: string
  headline: string
  name: string
  title: string
}

const TESTIMONIALS: Testimonial[] = [
  {
    headline: "Finally, a database client that doesn't feel overwhelming.",
    quote:
      "I mostly work with Postgres, and Queryon has been much easier to navigate than the heavier database tools I've used before. The interface is clean, and I can get to the data I need without digging through five different panels.",
    name: "Arjun Mehta",
    title: "Full-stack Developer",
  },
  {
    headline: "The table view is ridiculously convenient.",
    quote:
      "What I like most about Queryon is how quickly I can open a table, inspect the data, and start working with it. It feels more like a modern developer tool than an old-school database admin application.",
    name: "Rahul Sharma",
    title: "Backend Engineer",
  },
  {
    headline: "I stopped opening my browser every time I needed to check a database.",
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
  },
  {
    headline: "It feels like someone actually designed this for developers.",
    quote:
      "A lot of database tools feel like they were built around features first and usability second. Queryon feels different. The important stuff is easy to find, and the interface stays out of the way.",
    name: "Michael Chen",
    title: "Backend Developer",
  },
  {
    headline: "Great for quickly jumping into a database and figuring out what's going on.",
    quote:
      "I use database clients mostly for debugging and inspecting data. Queryon makes those quick tasks really easy without making me feel like I'm opening a full administration suite.",
    name: "Aman Verma",
    title: "Software Developer",
  },
]

function initials(name: string): string {
  return name
    .split(" ")
    .map((part) => part[0])
    .join("")
    .toUpperCase()
}

export function Testimonials() {
  return (
    <section className="mx-auto max-w-6xl px-6">
      <div className="mx-auto mb-12 max-w-2xl text-center">
        <h2 className="text-3xl font-medium tracking-tight text-balance sm:text-4xl">
          What people are saying
        </h2>
        <p className="mt-3 text-muted-foreground text-balance">
          Early feedback from developers using Queryon day to day.
        </p>
      </div>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {TESTIMONIALS.map((t) => (
          <figure
            key={t.name}
            className="flex flex-col gap-4 rounded-xl border bg-card p-6"
          >
            <blockquote className="flex-1">
              <p className="text-sm font-medium text-balance">&ldquo;{t.headline}&rdquo;</p>
              <p className="mt-2 text-sm text-muted-foreground">{t.quote}</p>
            </blockquote>
            <figcaption className="flex items-center gap-3">
              <span className="flex size-9 shrink-0 items-center justify-center rounded-full bg-muted text-xs font-medium text-muted-foreground">
                {initials(t.name)}
              </span>
              <div className="text-sm">
                <div className="font-medium">{t.name}</div>
                <div className="text-muted-foreground">{t.title}</div>
              </div>
            </figcaption>
          </figure>
        ))}
      </div>
    </section>
  )
}
