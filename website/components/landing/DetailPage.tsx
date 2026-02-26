import styles from '@/components/landing/landing.module.css'
import type { DetailPage as DetailPageType } from '@/data/landing'
import { LandingShell } from '@/components/landing/LandingShell'

export function DetailPage({ title, subtitle, summary, highlights, sections }: DetailPageType) {
  return (
    <LandingShell>
      <section className={styles.detailHero}>
        <div className={styles.detailHeroInner}>
          <div className={styles.detailMeta}>
            <h1 className={styles.detailTitle}>{title}</h1>
            <p className={styles.detailSubtitle}>{subtitle}</p>
            <div className={styles.detailHighlights}>
              {highlights.map((item) => (
                <span key={item}>{item}</span>
              ))}
            </div>
          </div>
          <div className={styles.detailSummary}>{summary}</div>
        </div>
      </section>

      {sections.map((section) => (
        <section key={section.title} className={styles.detailSection}>
          <div className={styles.detailSectionInner}>
            <div className={styles.detailCopy}>
              <h2>{section.title}</h2>
              <p>{section.description}</p>
            </div>
            <div className={styles.detailCards}>
              {section.cards.map((card) => (
                <div key={card.title} className={styles.detailCard}>
                  <span className={styles.cardLabel}>{card.label}</span>
                  <h3>{card.title}</h3>
                  <p>{card.description}</p>
                </div>
              ))}
            </div>
          </div>
        </section>
      ))}
    </LandingShell>
  )
}
