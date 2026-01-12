/**
 * Support Page
 *
 * Access help resources, submit tickets, and view support history
 */

import { MessageSquare, Book, Mail, ExternalLink, Send } from 'lucide-react'
import Link from 'next/link'
import styles from '../portal-pages.module.css'

export default function SupportPage() {
  return (
    <div className={styles.page}>
      <div className={styles.header}>
        <h1 className={styles.title}>Support</h1>
        <p className={styles.subtitle}>
          Get help with your Deka instance and manage support tickets
        </p>
      </div>

      {/* Quick Help */}
      <div className={styles.quickGrid}>
        <div className={styles.quickCard}>
          <div className={styles.iconPill}>
            <Book className="h-5 w-5" />
          </div>
          <div>
            <h3 className={styles.cardTitle}>Documentation</h3>
            <p className={styles.cardSubtitle}>Browse our guides and API references</p>
          </div>
          <Link href="/developers" className={styles.link}>
            View Docs
            <ExternalLink className="h-4 w-4" />
          </Link>
        </div>

        <div className={styles.quickCard}>
          <div className={styles.iconPill}>
            <MessageSquare className="h-5 w-5" />
          </div>
          <div>
            <h3 className={styles.cardTitle}>Community</h3>
            <p className={styles.cardSubtitle}>Join our Discord for live support</p>
          </div>
          <a
            href="https://discord.gg/deka"
            target="_blank"
            rel="noopener noreferrer"
            className={styles.link}
          >
            Join Discord
            <ExternalLink className="h-4 w-4" />
          </a>
        </div>

        <div className={styles.quickCard}>
          <div className={styles.iconPill}>
            <Mail className="h-5 w-5" />
          </div>
          <div>
            <h3 className={styles.cardTitle}>Email Support</h3>
            <p className={styles.cardSubtitle}>Reach us directly for account help</p>
          </div>
          <a href="mailto:support@deka.gg" className={styles.link}>
            support@deka.gg
            <ExternalLink className="h-4 w-4" />
          </a>
        </div>
      </div>

      {/* Submit Ticket */}
      <section className={styles.card}>
        <div className={styles.cardHeader}>
          <div>
            <h2 className={styles.cardTitle}>Submit a Support Ticket</h2>
            <p className={styles.cardSubtitle}>We usually respond within 24 hours</p>
          </div>
        </div>
        <form className={styles.form}>
          <div>
            <label htmlFor="subject" className={styles.label}>
              Subject
            </label>
            <input
              id="subject"
              type="text"
              placeholder="Brief description of your issue"
              className={styles.input}
            />
          </div>

          <div>
            <label htmlFor="category" className={styles.label}>
              Category
            </label>
            <select id="category" className={styles.select}>
              <option>Technical Issue</option>
              <option>Billing Question</option>
              <option>Feature Request</option>
              <option>General Inquiry</option>
            </select>
          </div>

          <div>
            <label htmlFor="message" className={styles.label}>
              Message
            </label>
            <textarea
              id="message"
              rows={6}
              placeholder="Describe your issue in detail..."
              className={styles.textarea}
            />
          </div>

          <button type="submit" className={styles.buttonPrimary}>
            <Send className="h-4 w-4" />
            Submit Ticket
          </button>
        </form>
      </section>

      {/* Recent Tickets */}
      <section className={styles.card}>
        <div className={styles.cardHeader}>
          <div>
            <h2 className={styles.cardTitle}>Recent Tickets</h2>
            <p className={styles.cardSubtitle}>Track your open requests</p>
          </div>
        </div>
        <div className={styles.dashedBox}>
          <MessageSquare className="h-10 w-10 mx-auto mb-3" />
          <p>No support tickets yet</p>
        </div>
      </section>
    </div>
  )
}
