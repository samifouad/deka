/**
 * Billing Page
 *
 * Manage subscriptions, payment methods, and invoices
 */

import { CreditCard, FileText, Calendar, DollarSign } from 'lucide-react'
import styles from '../portal-pages.module.css'

export default function BillingPage() {
  return (
    <div className={styles.page}>
      <div className={styles.header}>
        <h1 className={styles.title}>Billing</h1>
        <p className={styles.subtitle}>
          Manage your subscription, payment methods, and billing history
        </p>
      </div>

      {/* Current Plan */}
      <section className={styles.card}>
        <div className={styles.cardHeader}>
          <div>
            <h2 className={styles.cardTitle}>Current Plan</h2>
            <p className={styles.cardSubtitle}>Your active subscription</p>
          </div>
          <span className={styles.badge}>Active</span>
        </div>

        <div className={styles.statGrid}>
          <div className={styles.statItem}>
            <div className={styles.iconPill}>
              <DollarSign className="h-4 w-4" />
            </div>
            <div>
              <p className={styles.statLabel}>Monthly Cost</p>
              <p className={styles.statValue}>Coming Soon</p>
            </div>
          </div>

          <div className={styles.statItem}>
            <div className={styles.iconPill}>
              <Calendar className="h-4 w-4" />
            </div>
            <div>
              <p className={styles.statLabel}>Next Billing Date</p>
              <p className={styles.statValue}>TBD</p>
            </div>
          </div>

          <div className={styles.statItem}>
            <div className={styles.iconPill}>
              <CreditCard className="h-4 w-4" />
            </div>
            <div>
              <p className={styles.statLabel}>Payment Method</p>
              <p className={styles.statValue}>Not Set</p>
            </div>
          </div>
        </div>
      </section>

      {/* Payment Methods */}
      <section className={styles.card}>
        <div className={styles.cardHeader}>
          <div>
            <h2 className={styles.cardTitle}>Payment Methods</h2>
            <p className={styles.cardSubtitle}>Manage your saved cards</p>
          </div>
        </div>
        <div className={styles.dashedBox}>
          <CreditCard className="h-10 w-10 mx-auto mb-3" />
          <p className="mb-4">No payment methods added yet</p>
          <button className={styles.buttonPrimary}>Add Payment Method</button>
        </div>
      </section>

      {/* Billing History */}
      <section className={styles.card}>
        <div className={styles.cardHeader}>
          <div>
            <h2 className={styles.cardTitle}>Billing History</h2>
            <p className={styles.cardSubtitle}>Recent invoices and receipts</p>
          </div>
        </div>
        <div className={styles.dashedBox}>
          <FileText className="h-10 w-10 mx-auto mb-3" />
          <p>No invoices yet</p>
        </div>
      </section>

      {/* Info Note */}
      <div className={styles.notice}>
        <strong>Coming Soon:</strong> Billing features are currently in development. You'll be able to
        manage subscriptions, add payment methods, and view invoices once launched.
      </div>
    </div>
  )
}
