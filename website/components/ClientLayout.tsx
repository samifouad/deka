'use client'

import { usePathname } from "next/navigation"
import { UserProvider } from "@/context/user-context"
import { ThemeProvider } from "@/context/theme-context"
import { LangContextProvider } from "@/context/lang"
import { DebugTools } from "@/components/DebugTools"
import { MonacoLoader } from "@/components/MonacoLoader"
import Footer from "@/components/core/Footer"
import { languages } from "@/i18n"

interface ClientLayoutProps {
  children: React.ReactNode
  initialLang?: string
}

export function ClientLayout({ children, initialLang }: ClientLayoutProps) {
  const pathname = usePathname()
  const hideFooter = pathname?.startsWith('/help') ||
                     pathname?.startsWith('/cli') ||
                     pathname?.startsWith('/api') ||
                     pathname?.startsWith('/docs/') ||
                     pathname?.startsWith('/ui') ||
                     pathname?.startsWith('/portal')

  return (
    <ThemeProvider>
      <LangContextProvider initialLang={initialLang ?? (languages[0]?.code ?? 'en')}>
        <UserProvider>
          <MonacoLoader />
          {children}
          {!hideFooter && <Footer />}
        </UserProvider>
        <DebugTools />
      </LangContextProvider>
    </ThemeProvider>
  )
}
