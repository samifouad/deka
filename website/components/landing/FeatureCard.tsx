'use client'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'


interface FeatureCardProps {
  icon: React.ReactNode
  title: string
  description: string
}

export function FeatureCard({ icon, title, description }: FeatureCardProps) {
  return (
    <Card className="bg-card border-border backdrop-blur-sm hover:border-primary/40 transition-all duration-300 group">
      <CardHeader>
        <div className="w-12 h-12 bg-primary/20 border border-primary/30 rounded-lg flex items-center justify-center text-primary group-hover:bg-primary/30 group-hover:border-primary/50 transition-all duration-300">
          {icon}
        </div>
        <CardTitle className="text-foreground">{title}</CardTitle>
        <CardDescription className="text-muted-foreground">
          {description}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="h-1 bg-gradient-to-r from-transparent via-primary/20 to-transparent"></div>
      </CardContent>
    </Card>
  )
}