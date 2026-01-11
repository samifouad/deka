/**
 * Portal Home - Redirects to billing
 */

import { redirect } from 'next/navigation'

export default function PortalPage() {
  redirect('/portal/billing')
}
