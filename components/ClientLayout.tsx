'use client'

import { usePathname } from "next/navigation"
import { UserProvider } from "@/context/user-context"
import { ThemeProvider } from "@/context/theme-context"
import { DebugTools } from "@/components/DebugTools"
import { MonacoLoader } from "@/components/MonacoLoader"
import Footer from "@/components/core/Footer"

export function ClientLayout({ children }: { children: React.ReactNode }) {
  const pathname = usePathname()
  const hideFooter = pathname?.startsWith('/help') ||
                     pathname?.startsWith('/cli') ||
                     pathname?.startsWith('/api') ||
                     pathname?.startsWith('/ui') ||
                     pathname?.startsWith('/portal')

  return (
    <ThemeProvider>
      <UserProvider>
        <MonacoLoader />
        {children}
        {!hideFooter && <Footer />}
      </UserProvider>
      <DebugTools />
    </ThemeProvider>
  )
}
