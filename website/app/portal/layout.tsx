/**
 * Portal Layout
 *
 * Protected layout for authenticated users to access billing, support, and backups
 */

import { redirect } from 'next/navigation'
import { getCurrentUserData } from '@/actions/user'
import { PortalClientLayout } from './PortalClientLayout'

export default async function PortalLayout({
  children,
}: {
  children: React.ReactNode
}) {
  // Server-side auth check
  const user = await getCurrentUserData()

  if (!user) {
    redirect('/signin')
  }

  return <PortalClientLayout user={user}>{children}</PortalClientLayout>
}
