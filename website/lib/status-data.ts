export type ServiceStatus = 'operational' | 'degraded' | 'partial_outage' | 'major_outage'
export type IncidentSeverity = 'info' | 'warning' | 'critical'

export interface Service {
  id: string
  name: string
  description: string
  status: ServiceStatus
  uptime: number // percentage
}

export interface Incident {
  id: string
  title: string
  severity: IncidentSeverity
  status: 'investigating' | 'identified' | 'monitoring' | 'resolved'
  date: string
  updates: {
    timestamp: string
    message: string
    status: string
  }[]
}

export const services: Service[] = [
  {
    id: 'api',
    name: 'API',
    description: 'Core API endpoints',
    status: 'operational',
    uptime: 99.99
  },
  {
    id: 'edge',
    name: 'Edge Servers',
    description: 'Smart contract HTTP handlers',
    status: 'operational',
    uptime: 99.98
  },
  {
    id: 'database',
    name: 'Database',
    description: 'PostgreSQL blockchain data',
    status: 'operational',
    uptime: 99.99
  },
  {
    id: 'queue',
    name: 'Transaction Queue',
    description: 'Redis Streams mempool',
    status: 'operational',
    uptime: 99.97
  },
  {
    id: 'identity',
    name: 'Identity Service',
    description: 'Authentication & QR login',
    status: 'operational',
    uptime: 99.96
  },
  {
    id: 'notifications',
    name: 'Notifications',
    description: 'Push notification delivery',
    status: 'operational',
    uptime: 99.95
  },
  {
    id: 'mobile',
    name: 'Mobile App',
    description: 'iOS & Android applications',
    status: 'operational',
    uptime: 99.99
  },
  {
    id: 'website',
    name: 'Website',
    description: 'Landing page & dashboard',
    status: 'operational',
    uptime: 100.0
  }
]

export const incidents: Incident[] = [
  {
    id: 'inc_003',
    title: 'Scheduled Maintenance - Database Migration',
    severity: 'info',
    status: 'resolved',
    date: '2024-11-10',
    updates: [
      {
        timestamp: '2024-11-10T18:00:00Z',
        message: 'Maintenance completed successfully. All services are operational.',
        status: 'resolved'
      },
      {
        timestamp: '2024-11-10T15:00:00Z',
        message: 'Maintenance in progress. Brief API slowdowns expected.',
        status: 'monitoring'
      },
      {
        timestamp: '2024-11-10T14:45:00Z',
        message: 'Starting scheduled database migration. Expected duration: 3 hours.',
        status: 'identified'
      }
    ]
  },
  {
    id: 'inc_002',
    title: 'Intermittent API Slowdowns',
    severity: 'warning',
    status: 'resolved',
    date: '2024-11-08',
    updates: [
      {
        timestamp: '2024-11-08T16:30:00Z',
        message: 'All systems are operating normally. Issue has been fully resolved.',
        status: 'resolved'
      },
      {
        timestamp: '2024-11-08T15:15:00Z',
        message: 'We have applied a fix and are monitoring the situation. Response times have improved.',
        status: 'monitoring'
      },
      {
        timestamp: '2024-11-08T14:45:00Z',
        message: 'We have identified the cause as a database query optimization issue and are deploying a fix.',
        status: 'identified'
      },
      {
        timestamp: '2024-11-08T14:30:00Z',
        message: 'We are investigating reports of slower than usual API response times.',
        status: 'investigating'
      }
    ]
  },
  {
    id: 'inc_001',
    title: 'Redis Queue Connection Issues',
    severity: 'critical',
    status: 'resolved',
    date: '2024-11-05',
    updates: [
      {
        timestamp: '2024-11-05T12:00:00Z',
        message: 'All services restored. Transaction processing is back to normal.',
        status: 'resolved'
      },
      {
        timestamp: '2024-11-05T11:30:00Z',
        message: 'Fix deployed. Monitoring transaction queue for stability.',
        status: 'monitoring'
      },
      {
        timestamp: '2024-11-05T11:00:00Z',
        message: 'Issue identified: Redis connection pool exhaustion. Deploying fix.',
        status: 'identified'
      },
      {
        timestamp: '2024-11-05T10:45:00Z',
        message: 'We are investigating issues with transaction processing delays.',
        status: 'investigating'
      }
    ]
  }
]

export function getOverallStatus(): ServiceStatus {
  const statuses = services.map(s => s.status)

  if (statuses.includes('major_outage')) return 'major_outage'
  if (statuses.includes('partial_outage')) return 'partial_outage'
  if (statuses.includes('degraded')) return 'degraded'
  return 'operational'
}

export function getStatusColor(status: ServiceStatus): string {
  switch (status) {
    case 'operational':
      return 'text-green-500'
    case 'degraded':
      return 'text-yellow-500'
    case 'partial_outage':
      return 'text-orange-500'
    case 'major_outage':
      return 'text-red-500'
  }
}

export function getStatusBgColor(status: ServiceStatus): string {
  switch (status) {
    case 'operational':
      return 'bg-green-500/10'
    case 'degraded':
      return 'bg-yellow-500/10'
    case 'partial_outage':
      return 'bg-orange-500/10'
    case 'major_outage':
      return 'bg-red-500/10'
  }
}

export function getStatusText(status: ServiceStatus): string {
  switch (status) {
    case 'operational':
      return 'All Systems Operational'
    case 'degraded':
      return 'Degraded Performance'
    case 'partial_outage':
      return 'Partial Outage'
    case 'major_outage':
      return 'Major Outage'
  }
}
