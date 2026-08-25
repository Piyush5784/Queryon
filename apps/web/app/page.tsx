import { EngineMarquee } from "../components/landing/EngineMarquee"
import { FeatureDeepDives } from "../components/landing/FeatureDeepDives"
import { FeatureGrid } from "../components/landing/FeatureGrid"
import { Hero } from "../components/landing/Hero"
import { Navbar } from "../components/landing/Navbar"
import { ReadyToGetStarted } from "../components/landing/ReadyToGetStarted"
import { SiteFooter } from "../components/landing/SiteFooter"
import { TextReveal } from "../components/landing/TextReveal"

export default function LandingPage() {
  return (
    <div className="flex min-h-svh flex-col">
      <Navbar />
      <main className="flex-1">
        <Hero />
        <EngineMarquee />

        <div className="space-y-20 sm:space-y-[120px] lg:space-y-[180px]">
          <FeatureDeepDives />
          <FeatureGrid />
          <TextReveal>
            Queryon isn&apos;t a smaller DBeaver or a rebranded pgAdmin. It&apos;s a client
            written for people who spend their whole day inside a database.
          </TextReveal>
          <ReadyToGetStarted />
        </div>
      </main>
      <SiteFooter />
    </div>
  )
}
