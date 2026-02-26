import styles from '@/components/landing/landing.module.css'
import type { FeatureCard } from '@/data/landing'

export function HorizontalScroll({ title, description, cards }: { title: string; description: string; cards: FeatureCard[] }) {
  return (
    <section className={styles.scrollSection}>
      <div className={styles.scrollInner}>
        <div className={styles.scrollHeader}>
          <div>
            <h2>{title}</h2>
            <p>{description}</p>
          </div>
        </div>
      </div>
      <div className={styles.scrollTrack}>
        {cards.map((card) => (
          <div key={card.title} className={styles.scrollCard}>
            <span className={styles.cardLabel}>{card.label}</span>
            <h3>{card.title}</h3>
            <p>{card.description}</p>
          </div>
        ))}
      </div>
    </section>
  )
}
