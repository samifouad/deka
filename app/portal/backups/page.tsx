/**
 * Backups Page
 *
 * Manage automated backups of Deka instances
 */

import { Database, Download, Settings, Clock, HardDrive, CheckCircle } from 'lucide-react'
import { Card, CardContent } from '@/components/ui/card'

export default function BackupsPage() {
  return (
    <div className="space-y-8">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-foreground mb-2">Backups</h1>
        <p className="text-muted-foreground">
          Manage automated backups and restore points for your Deka instances
        </p>
      </div>

      {/* Backup Status */}
      <Card className="bg-card border-border">
        <CardContent className="pt-6">
          <div className="flex items-start justify-between mb-6">
            <div>
              <h2 className="text-xl font-semibold text-foreground mb-1">Backup Status</h2>
              <p className="text-sm text-muted-foreground">Overview of your backup configuration</p>
            </div>
            <div className="px-3 py-1 bg-green-500/10 border border-green-500/30 rounded-full flex items-center gap-2">
              <CheckCircle className="w-4 h-4 text-green-500" />
              <span className="text-sm font-medium text-green-500">Healthy</span>
            </div>
          </div>

          <div className="grid md:grid-cols-3 gap-6">
            <div className="flex items-start gap-3">
              <div className="p-2 bg-secondary rounded-lg">
                <Clock className="w-5 h-5 text-primary" />
              </div>
              <div>
                <p className="text-sm text-muted-foreground">Last Backup</p>
                <p className="text-lg font-semibold text-foreground">N/A</p>
              </div>
            </div>

            <div className="flex items-start gap-3">
              <div className="p-2 bg-secondary rounded-lg">
                <Database className="w-5 h-5 text-primary" />
              </div>
              <div>
                <p className="text-sm text-muted-foreground">Total Backups</p>
                <p className="text-lg font-semibold text-foreground">0</p>
              </div>
            </div>

            <div className="flex items-start gap-3">
              <div className="p-2 bg-secondary rounded-lg">
                <HardDrive className="w-5 h-5 text-primary" />
              </div>
              <div>
                <p className="text-sm text-muted-foreground">Storage Used</p>
                <p className="text-lg font-semibold text-foreground">0 GB</p>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Backup Settings */}
      <Card className="bg-card border-border">
        <CardContent className="pt-6">
          <div className="flex items-center justify-between mb-6">
            <div>
              <h2 className="text-xl font-semibold text-foreground mb-1">Backup Settings</h2>
              <p className="text-sm text-muted-foreground">Configure automated backup schedule</p>
            </div>
            <button className="flex items-center gap-2 px-4 py-2 bg-secondary hover:bg-secondary/80 text-foreground rounded-lg transition-colors">
              <Settings className="w-4 h-4" />
              Configure
            </button>
          </div>

          <div className="space-y-4">
            <div className="flex items-center justify-between p-4 bg-secondary/50 rounded-lg">
              <div>
                <p className="font-medium text-foreground">Automatic Backups</p>
                <p className="text-sm text-muted-foreground">Daily backups at 2:00 AM UTC</p>
              </div>
              <div className="flex items-center">
                <label className="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" className="sr-only peer" disabled />
                  <div className="w-11 h-6 bg-muted peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary"></div>
                </label>
              </div>
            </div>

            <div className="flex items-center justify-between p-4 bg-secondary/50 rounded-lg">
              <div>
                <p className="font-medium text-foreground">Retention Period</p>
                <p className="text-sm text-muted-foreground">Keep backups for 30 days</p>
              </div>
              <span className="text-sm text-muted-foreground">30 days</span>
            </div>

            <div className="flex items-center justify-between p-4 bg-secondary/50 rounded-lg">
              <div>
                <p className="font-medium text-foreground">Encryption</p>
                <p className="text-sm text-muted-foreground">End-to-end encrypted backups</p>
              </div>
              <div className="px-2 py-1 bg-green-500/10 border border-green-500/30 rounded">
                <span className="text-xs font-medium text-green-500">Enabled</span>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Backup History */}
      <Card className="bg-card border-border">
        <CardContent className="pt-6">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-xl font-semibold text-foreground">Backup History</h2>
            <button className="flex items-center gap-2 px-4 py-2 bg-primary hover:bg-primary/90 text-primary-foreground rounded-lg transition-colors">
              <Database className="w-4 h-4" />
              Create Backup Now
            </button>
          </div>
          <div className="text-center py-12 border-2 border-dashed border-border rounded-lg">
            <Database className="w-12 h-12 text-muted-foreground mx-auto mb-3" />
            <p className="text-muted-foreground mb-2">No backups available</p>
            <p className="text-sm text-muted-foreground">Create your first backup to get started</p>
          </div>
        </CardContent>
      </Card>

      {/* Info Note */}
      <div className="p-4 bg-blue-500/10 border border-blue-500/30 rounded-lg">
        <p className="text-sm text-blue-400">
          <strong>Coming Soon:</strong> Automated backup functionality is currently in development.
          You'll be able to schedule backups, restore from snapshots, and manage retention policies once launched.
        </p>
      </div>
    </div>
  )
}
