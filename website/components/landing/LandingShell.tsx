import { ReactNode } from 'react'
import { Navbar } from '@/components/landing/Navbar'
import styles from '@/components/landing/landing.module.css'

export function LandingShell({ children }: { children: ReactNode }) {
  return (
    <div className={styles.page}>
      <Navbar mode="fixed" />
      <main className={styles.main}>{children}</main>
    </div>
  )
}
