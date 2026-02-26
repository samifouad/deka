import { Plus } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger
} from '@/components/ui/dialog'
import styles from '@/components/landing/landing.module.css'
import type { ExpandableCard } from '@/data/landing'

export function ExpandableCards({ title, description, cards }: { title: string; description: string; cards: ExpandableCard[] }) {
  return (
    <section className={styles.expandSection}>
      <div className={styles.expandHeader}>
        <h2>{title}</h2>
        <p>{description}</p>
      </div>
      <div className={styles.expandGrid}>
        {cards.map((card) => (
          <Dialog key={card.title}>
            <DialogTrigger asChild>
              <button type="button" className={styles.expandCard}>
                <div>
                  <span className={styles.cardLabel}>{card.label}</span>
                  <h3>{card.title}</h3>
                  <p>{card.description}</p>
                </div>
                <span className={styles.plusButton}>
                  <Plus className="h-4 w-4" />
                </span>
              </button>
            </DialogTrigger>
            <DialogContent className={styles.expandModal}>
              <DialogHeader className={styles.expandModalHeader}>
                <DialogTitle className={styles.expandModalTitle}>{card.title}</DialogTitle>
                <DialogDescription className={styles.expandModalDescription}>
                  {card.longDescription}
                </DialogDescription>
              </DialogHeader>
              <div className={styles.expandModalBody}>
                {card.points.map((point) => (
                  <div key={point.title} className={styles.expandModalCard}>
                    <span className={styles.cardLabel}>{point.label}</span>
                    <h3>{point.title}</h3>
                    <p>{point.description}</p>
                  </div>
                ))}
              </div>
            </DialogContent>
          </Dialog>
        ))}
      </div>
    </section>
  )
}
