/**
 * Billing Page
 *
 * Manage subscriptions, payment methods, and invoices
 */

import { CreditCard, FileText, Calendar, DollarSign } from 'lucide-react'
import { Card, CardContent } from '@/components/ui/card'

export default function BillingPage() {
  return (
    <div className="space-y-8">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-foreground mb-2">Billing</h1>
        <p className="text-muted-foreground">
          Manage your subscription, payment methods, and billing history
        </p>
      </div>

      {/* Current Plan */}
      <Card className="bg-card border-border">
        <CardContent className="pt-6">
          <div className="flex items-start justify-between mb-6">
            <div>
              <h2 className="text-xl font-semibold text-foreground mb-1">Current Plan</h2>
              <p className="text-sm text-muted-foreground">Your active subscription</p>
            </div>
            <div className="px-3 py-1 bg-primary/10 border border-primary/30 rounded-full">
              <span className="text-sm font-medium text-primary">Active</span>
            </div>
          </div>

          <div className="grid md:grid-cols-3 gap-6">
            <div className="flex items-start gap-3">
              <div className="p-2 bg-secondary rounded-lg">
                <DollarSign className="w-5 h-5 text-primary" />
              </div>
              <div>
                <p className="text-sm text-muted-foreground">Monthly Cost</p>
                <p className="text-lg font-semibold text-foreground">Coming Soon</p>
              </div>
            </div>

            <div className="flex items-start gap-3">
              <div className="p-2 bg-secondary rounded-lg">
                <Calendar className="w-5 h-5 text-primary" />
              </div>
              <div>
                <p className="text-sm text-muted-foreground">Next Billing Date</p>
                <p className="text-lg font-semibold text-foreground">TBD</p>
              </div>
            </div>

            <div className="flex items-start gap-3">
              <div className="p-2 bg-secondary rounded-lg">
                <CreditCard className="w-5 h-5 text-primary" />
              </div>
              <div>
                <p className="text-sm text-muted-foreground">Payment Method</p>
                <p className="text-lg font-semibold text-foreground">Not Set</p>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Payment Methods */}
      <Card className="bg-card border-border">
        <CardContent className="pt-6">
          <h2 className="text-xl font-semibold text-foreground mb-4">Payment Methods</h2>
          <div className="text-center py-12 border-2 border-dashed border-border rounded-lg">
            <CreditCard className="w-12 h-12 text-muted-foreground mx-auto mb-3" />
            <p className="text-muted-foreground mb-4">No payment methods added yet</p>
            <button className="px-4 py-2 bg-primary hover:bg-primary/90 text-primary-foreground rounded-lg transition-colors">
              Add Payment Method
            </button>
          </div>
        </CardContent>
      </Card>

      {/* Billing History */}
      <Card className="bg-card border-border">
        <CardContent className="pt-6">
          <h2 className="text-xl font-semibold text-foreground mb-4">Billing History</h2>
          <div className="text-center py-12 border-2 border-dashed border-border rounded-lg">
            <FileText className="w-12 h-12 text-muted-foreground mx-auto mb-3" />
            <p className="text-muted-foreground">No invoices yet</p>
          </div>
        </CardContent>
      </Card>

      {/* Info Note */}
      <div className="p-4 bg-blue-500/10 border border-blue-500/30 rounded-lg">
        <p className="text-sm text-blue-400">
          <strong>Coming Soon:</strong> Billing features are currently in development.
          You'll be able to manage subscriptions, add payment methods, and view invoices once launched.
        </p>
      </div>
    </div>
  )
}
