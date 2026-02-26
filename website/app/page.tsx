import { LandingShell } from '@/components/landing/LandingShell'
import { LandingHero } from '@/components/landing/LandingHero'
import { FeatureSection } from '@/components/landing/FeatureSection'
import { HorizontalScroll } from '@/components/landing/HorizontalScroll'
import { ExpandableCards } from '@/components/landing/ExpandableCards'
import { featureSections, scrollShowcase, expandShowcase, modelShowcase } from '@/data/landing'

export default function HomePage() {
  return (
    <LandingShell>
      <LandingHero />
      <ExpandableCards {...expandShowcase} />
      <HorizontalScroll {...scrollShowcase} />
      <HorizontalScroll {...modelShowcase} />
      {featureSections.map((section, index) => (
        <FeatureSection
          key={section.id}
          section={section}
          isAlt={index % 2 === 1}
        />
      ))}
    </LandingShell>
  )
}
