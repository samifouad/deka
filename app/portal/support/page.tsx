/**
 * Support Page
 *
 * Access help resources, submit tickets, and view support history
 */

import { MessageSquare, Book, Mail, ExternalLink, Send } from 'lucide-react'
import { Card, CardContent } from '@/components/ui/card'
import Link from 'next/link'

export default function SupportPage() {
  return (
    <div className="space-y-8">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-foreground mb-2">Support</h1>
        <p className="text-muted-foreground">
          Get help with your Deka instance and manage support tickets
        </p>
      </div>

      {/* Quick Help */}
      <div className="grid md:grid-cols-3 gap-6">
        <Card className="bg-card border-border hover:border-primary/30 transition-colors">
          <CardContent className="pt-6">
            <div className="p-3 bg-primary/10 rounded-lg w-fit mb-4">
              <Book className="w-6 h-6 text-primary" />
            </div>
            <h3 className="text-lg font-semibold text-foreground mb-2">Documentation</h3>
            <p className="text-sm text-muted-foreground mb-4">
              Browse our comprehensive guides and API references
            </p>
            <Link
              href="/developers"
              className="inline-flex items-center gap-2 text-sm text-primary hover:text-primary/80 transition-colors"
            >
              View Docs
              <ExternalLink className="w-4 h-4" />
            </Link>
          </CardContent>
        </Card>

        <Card className="bg-card border-border hover:border-primary/30 transition-colors">
          <CardContent className="pt-6">
            <div className="p-3 bg-primary/10 rounded-lg w-fit mb-4">
              <MessageSquare className="w-6 h-6 text-primary" />
            </div>
            <h3 className="text-lg font-semibold text-foreground mb-2">Community</h3>
            <p className="text-sm text-muted-foreground mb-4">
              Join our Discord community for discussions and help
            </p>
            <a
              href="https://discord.gg/deka"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-2 text-sm text-primary hover:text-primary/80 transition-colors"
            >
              Join Discord
              <ExternalLink className="w-4 h-4" />
            </a>
          </CardContent>
        </Card>

        <Card className="bg-card border-border hover:border-primary/30 transition-colors">
          <CardContent className="pt-6">
            <div className="p-3 bg-primary/10 rounded-lg w-fit mb-4">
              <Mail className="w-6 h-6 text-primary" />
            </div>
            <h3 className="text-lg font-semibold text-foreground mb-2">Email Support</h3>
            <p className="text-sm text-muted-foreground mb-4">
              Reach out to our team directly via email
            </p>
            <a
              href="mailto:support@deka.gg"
              className="inline-flex items-center gap-2 text-sm text-primary hover:text-primary/80 transition-colors"
            >
              support@deka.gg
              <ExternalLink className="w-4 h-4" />
            </a>
          </CardContent>
        </Card>
      </div>

      {/* Submit Ticket */}
      <Card className="bg-card border-border">
        <CardContent className="pt-6">
          <h2 className="text-xl font-semibold text-foreground mb-4">Submit a Support Ticket</h2>
          <form className="space-y-4">
            <div>
              <label htmlFor="subject" className="block text-sm font-medium text-foreground mb-2">
                Subject
              </label>
              <input
                id="subject"
                type="text"
                placeholder="Brief description of your issue"
                className="w-full px-4 py-3 bg-input border border-border rounded-lg text-foreground placeholder-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary"
              />
            </div>

            <div>
              <label htmlFor="category" className="block text-sm font-medium text-foreground mb-2">
                Category
              </label>
              <select
                id="category"
                className="w-full px-4 py-3 bg-input border border-border rounded-lg text-foreground focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary"
              >
                <option>Technical Issue</option>
                <option>Billing Question</option>
                <option>Feature Request</option>
                <option>General Inquiry</option>
              </select>
            </div>

            <div>
              <label htmlFor="message" className="block text-sm font-medium text-foreground mb-2">
                Message
              </label>
              <textarea
                id="message"
                rows={6}
                placeholder="Describe your issue in detail..."
                className="w-full px-4 py-3 bg-input border border-border rounded-lg text-foreground placeholder-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary resize-none"
              />
            </div>

            <button
              type="submit"
              className="flex items-center gap-2 px-6 py-3 bg-primary hover:bg-primary/90 text-primary-foreground rounded-lg transition-colors"
            >
              <Send className="w-4 h-4" />
              Submit Ticket
            </button>
          </form>
        </CardContent>
      </Card>

      {/* Recent Tickets */}
      <Card className="bg-card border-border">
        <CardContent className="pt-6">
          <h2 className="text-xl font-semibold text-foreground mb-4">Recent Tickets</h2>
          <div className="text-center py-12 border-2 border-dashed border-border rounded-lg">
            <MessageSquare className="w-12 h-12 text-muted-foreground mx-auto mb-3" />
            <p className="text-muted-foreground">No support tickets yet</p>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
