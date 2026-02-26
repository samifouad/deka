import { getRequestLang } from '@/lib/i18n-server'
import { loadRuntimeDocs } from '@/lib/runtime-docs-server'
import { getRuntimeSidebar } from '@/lib/runtime-docs'
import { RuntimeDocsShell } from '@/components/docs/RuntimeDocsShell'

export default async function DocsPHPLayout({
  children,
}: {
  children: React.ReactNode
}) {
  const lang = await getRequestLang()
  const docs = loadRuntimeDocs(lang)
  const sections = getRuntimeSidebar('php', docs)

  return (
    <RuntimeDocsShell
      language="php"
      sections={sections}
      searchPlaceholder="Search PHP runtime docs..."
    >
      {children}
    </RuntimeDocsShell>
  )
}
