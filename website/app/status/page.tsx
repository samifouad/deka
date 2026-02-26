'use client'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { BlueprintGrid } from '@/components/landing/BlueprintGrid'
import { Navbar } from '@/components/landing/Navbar'
import {
  services,
  incidents,
  getOverallStatus,
  getStatusColor,
  getStatusBgColor,
  getStatusText
} from '@/lib/status-data'
import { CheckCircle2, AlertCircle, AlertTriangle, XCircle, Clock } from 'lucide-react'

function StatusIcon({ status }: { status: string }) {
  switch (status) {
    case 'operational':
      return <CheckCircle2 className="w-5 h-5 text-green-500" />
    case 'degraded':
      return <AlertTriangle className="w-5 h-5 text-yellow-500" />
    case 'partial_outage':
      return <AlertCircle className="w-5 h-5 text-orange-500" />
    case 'major_outage':
      return <XCircle className="w-5 h-5 text-red-500" />
    default:
      return <CheckCircle2 className="w-5 h-5 text-green-500" />
  }
}

function SeverityBadge({ severity }: { severity: string }) {
  const colors = {
    info: 'bg-blue-500/10 text-blue-500',
    warning: 'bg-yellow-500/10 text-yellow-500',
    critical: 'bg-red-500/10 text-red-500'
  }

  return (
    <span className={`text-xs px-2 py-1 rounded-full ${colors[severity as keyof typeof colors]}`}>
      {severity.toUpperCase()}
    </span>
  )
}

export default function StatusPage() {
  const overallStatus = getOverallStatus()

  return (
    <div className="min-h-screen bg-background text-foreground overflow-hidden">
      <BlueprintGrid />
      <Navbar />

      {/* Header */}
      <section className="relative px-4 py-24 lg:px-8">
        <div className="absolute inset-0 bg-gradient-to-br from-primary/10 via-background/20 to-primary/5 backdrop-blur-sm"></div>

        <div className="relative mx-auto max-w-7xl text-center">
          <h1 className="text-5xl lg:text-6xl font-bold text-foreground mb-6">
            System Status
          </h1>
          <p className="text-xl text-muted-foreground max-w-2xl mx-auto mb-8">
            Real-time status and incident updates for all tana services.
          </p>

          {/* Overall Status Banner */}
          <Card className={`${getStatusBgColor(overallStatus)} border-border backdrop-blur-sm max-w-md mx-auto`}>
            <CardContent className="pt-6">
              <div className="flex items-center justify-center gap-3">
                <StatusIcon status={overallStatus} />
                <span className={`text-lg font-bold ${getStatusColor(overallStatus)}`}>
                  {getStatusText(overallStatus)}
                </span>
              </div>
            </CardContent>
          </Card>
        </div>
      </section>

      {/* Services Status */}
      <section className="relative px-4 py-16 lg:px-8">
        <div className="mx-auto max-w-7xl">
          <h2 className="text-3xl font-bold text-foreground mb-8">Service Status</h2>

          <div className="space-y-3">
            {services.map((service) => (
              <Card key={service.id} className="bg-card border-border backdrop-blur-sm">
                <CardContent className="pt-6">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-4 flex-1">
                      <StatusIcon status={service.status} />
                      <div>
                        <h3 className="font-semibold text-foreground">{service.name}</h3>
                        <p className="text-sm text-muted-foreground">{service.description}</p>
                      </div>
                    </div>

                    <div className="text-right">
                      <div className={`font-semibold ${getStatusColor(service.status)} capitalize`}>
                        {service.status.replace('_', ' ')}
                      </div>
                      <div className="text-sm text-muted-foreground">
                        {service.uptime}% uptime
                      </div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      </section>

      {/* Uptime Chart Placeholder */}
      <section className="relative px-4 py-16 lg:px-8 bg-secondary/30">
        <div className="mx-auto max-w-7xl">
          <h2 className="text-3xl font-bold text-foreground mb-8">Uptime History (Last 90 Days)</h2>

          <Card className="bg-card border-border backdrop-blur-sm">
            <CardContent className="pt-6">
              <div className="flex items-center justify-center gap-1 h-16">
                {/* Simple bar chart representation */}
                {Array.from({ length: 90 }).map((_, i) => (
                  <div
                    key={i}
                    className={`flex-1 ${
                      Math.random() > 0.01 ? 'bg-green-500' : 'bg-red-500'
                    } h-full rounded-sm`}
                    title={`Day ${i + 1}`}
                  />
                ))}
              </div>
              <div className="flex justify-between text-sm text-muted-foreground mt-4">
                <span>90 days ago</span>
                <span className="flex items-center gap-2">
                  <span className="w-3 h-3 bg-green-500 rounded-sm" /> Operational
                  <span className="w-3 h-3 bg-red-500 rounded-sm ml-3" /> Downtime
                </span>
                <span>Today</span>
              </div>
            </CardContent>
          </Card>
        </div>
      </section>

      {/* Incident History */}
      <section className="relative px-4 py-16 lg:px-8">
        <div className="mx-auto max-w-7xl">
          <h2 className="text-3xl font-bold text-foreground mb-8">Recent Incidents</h2>

          <div className="space-y-6">
            {incidents.map((incident) => (
              <Card key={incident.id} className="bg-card border-border backdrop-blur-sm">
                <CardHeader>
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex-1">
                      <div className="flex items-center gap-3 mb-2">
                        <SeverityBadge severity={incident.severity} />
                        <span className={`text-sm font-semibold capitalize ${
                          incident.status === 'resolved' ? 'text-green-500' : 'text-yellow-500'
                        }`}>
                          {incident.status.replace('_', ' ')}
                        </span>
                      </div>
                      <CardTitle className="text-foreground">{incident.title}</CardTitle>
                      <CardDescription className="text-muted-foreground">
                        {new Date(incident.date).toLocaleDateString('en-US', {
                          month: 'long',
                          day: 'numeric',
                          year: 'numeric'
                        })}
                      </CardDescription>
                    </div>
                  </div>
                </CardHeader>

                <CardContent>
                  <div className="space-y-4">
                    {incident.updates.map((update, index) => (
                      <div key={index} className="flex gap-4">
                        <div className="flex flex-col items-center">
                          <div className="w-8 h-8 bg-primary/10 rounded-full flex items-center justify-center flex-shrink-0">
                            <Clock className="w-4 h-4 text-primary" />
                          </div>
                          {index !== incident.updates.length - 1 && (
                            <div className="w-px h-full bg-border mt-2" />
                          )}
                        </div>

                        <div className="flex-1 pb-4">
                          <div className="flex items-center gap-3 mb-1">
                            <span className="text-sm font-semibold text-muted-foreground">
                              {new Date(update.timestamp).toLocaleString('en-US', {
                                month: 'short',
                                day: 'numeric',
                                hour: '2-digit',
                                minute: '2-digit'
                              })}
                            </span>
                            <span className={`text-xs px-2 py-0.5 rounded-full capitalize ${
                              update.status === 'resolved'
                                ? 'bg-green-500/10 text-green-500'
                                : update.status === 'monitoring'
                                ? 'bg-blue-500/10 text-blue-500'
                                : 'bg-yellow-500/10 text-yellow-500'
                            }`}>
                              {update.status}
                            </span>
                          </div>
                          <p className="text-muted-foreground text-sm">{update.message}</p>
                        </div>
                      </div>
                    ))}
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      </section>

      {/* Subscribe to Updates */}
      <section className="relative px-4 py-16 lg:px-8 bg-secondary/30">
        <div className="mx-auto max-w-4xl text-center">
          <h2 className="text-3xl font-bold text-foreground mb-4">
            Get Status Updates
          </h2>
          <p className="text-lg text-muted-foreground mb-8">
            Subscribe to receive notifications when incidents occur or are resolved.
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center max-w-md mx-auto">
            <input
              type="email"
              placeholder="Enter your email"
              className="flex-1 px-4 py-2 bg-background border border-border rounded-lg focus:outline-none focus:border-primary text-foreground"
            />
            <button className="px-6 py-2 bg-primary hover:bg-primary/90 text-primary-foreground rounded-lg font-semibold transition-colors">
              Subscribe
            </button>
          </div>
        </div>
      </section>

    </div>
  )
}
