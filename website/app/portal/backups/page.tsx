/**
 * Backups Page
 *
 * Manage automated backups of Deka instances
 */

import { Database, Download, Settings, Clock, HardDrive, CheckCircle } from 'lucide-react'
import styles from '../portal-pages.module.css'

export default function BackupsPage() {
  return (
    <div className={styles.page}>
      <div className={styles.header}>
        <h1 className={styles.title}>Backups</h1>
        <p className={styles.subtitle}>
          Manage automated backups and restore points for your Deka instances
        </p>
      </div>

      {/* Backup Status */}
      <section className={styles.card}>
        <div className={styles.cardHeader}>
          <div>
            <h2 className={styles.cardTitle}>Backup Status</h2>
            <p className={styles.cardSubtitle}>Overview of your backup configuration</p>
          </div>
          <span className={styles.badgeSuccess}>
            <CheckCircle className="h-4 w-4" />
            Healthy
          </span>
        </div>

        <div className={styles.statGrid}>
          <div className={styles.statItem}>
            <div className={styles.iconPill}>
              <Clock className="h-4 w-4" />
            </div>
            <div>
              <p className={styles.statLabel}>Last Backup</p>
              <p className={styles.statValue}>N/A</p>
            </div>
          </div>

          <div className={styles.statItem}>
            <div className={styles.iconPill}>
              <Database className="h-4 w-4" />
            </div>
            <div>
              <p className={styles.statLabel}>Total Backups</p>
              <p className={styles.statValue}>0</p>
            </div>
          </div>

          <div className={styles.statItem}>
            <div className={styles.iconPill}>
              <HardDrive className="h-4 w-4" />
            </div>
            <div>
              <p className={styles.statLabel}>Storage Used</p>
              <p className={styles.statValue}>0 GB</p>
            </div>
          </div>
        </div>
      </section>

      {/* Backup Settings */}
      <section className={styles.card}>
        <div className={styles.cardHeader}>
          <div>
            <h2 className={styles.cardTitle}>Backup Settings</h2>
            <p className={styles.cardSubtitle}>Configure automated backup schedule</p>
          </div>
          <button className={styles.buttonSecondary}>
            <Settings className="h-4 w-4" />
            Configure
          </button>
        </div>

        <div className="grid gap-12">
          <div className={styles.statItem}>
            <div className={styles.iconPill}>
              <Clock className="h-4 w-4" />
            </div>
            <div>
              <p className={styles.statLabel}>Automatic Backups</p>
              <p className={styles.statValue}>Daily backups at 2:00 AM UTC</p>
            </div>
          </div>

          <div className={styles.statItem}>
            <div className={styles.iconPill}>
              <HardDrive className="h-4 w-4" />
            </div>
            <div>
              <p className={styles.statLabel}>Retention Period</p>
              <p className={styles.statValue}>Keep backups for 30 days</p>
            </div>
          </div>

          <div className={styles.statItem}>
            <div className={styles.iconPill}>
              <Database className="h-4 w-4" />
            </div>
            <div>
              <p className={styles.statLabel}>Encryption</p>
              <p className={styles.statValue}>End-to-end encrypted backups</p>
            </div>
          </div>
        </div>
      </section>

      {/* Backup History */}
      <section className={styles.card}>
        <div className={styles.cardHeader}>
          <div>
            <h2 className={styles.cardTitle}>Backup History</h2>
            <p className={styles.cardSubtitle}>Recent snapshots and restores</p>
          </div>
          <button className={styles.buttonPrimary}>
            <Database className="h-4 w-4" />
            Create Backup Now
          </button>
        </div>
        <div className={styles.dashedBox}>
          <Database className="h-10 w-10 mx-auto mb-3" />
          <p className="mb-2">No backups available</p>
          <p>Create your first backup to get started</p>
        </div>
      </section>

      {/* Info Note */}
      <div className={styles.notice}>
        <strong>Coming Soon:</strong> Automated backup functionality is currently in development. You'll
        be able to schedule backups, restore from snapshots, and manage retention policies once launched.
      </div>
    </div>
  )
}
