import styles from '@/components/landing/landing.module.css'
import type { FeatureSection as FeatureSectionType } from '@/data/landing'

export function FeatureSection({ section, isAlt }: { section: FeatureSectionType; isAlt: boolean }) {
  return (
    <section
      id={section.id}
      className={`${styles.section} ${isAlt ? styles.sectionAlt : ''}`}
    >
      <div className={styles.sectionInner}>
        <div className={styles.sectionCopy}>
          <span className={styles.sectionEyebrow}>{section.eyebrow}</span>
          <h2>{section.title}</h2>
          <p>{section.description}</p>
        </div>
        <div className={styles.cards}>
          {section.cards.map((card, index) => (
            <div
              key={card.title}
              className={`${styles.featureCard} ${styles.reveal}`}
              style={{ animationDelay: `${index * 0.12 + 0.05}s` }}
            >
              <span className={styles.cardLabel}>{card.label}</span>
              <h3>{card.title}</h3>
              <p>{card.description}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  )
}
