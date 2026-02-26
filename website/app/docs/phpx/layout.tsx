import { getRequestLang } from '@/lib/i18n-server'
import { loadRuntimeDocs } from '@/lib/runtime-docs-server'
import { getRuntimeSidebar } from '@/lib/runtime-docs'
import { RuntimeDocsShell } from '@/components/docs/RuntimeDocsShell'

export default async function DocsJSLayout({
  children,
}: {
  children: React.ReactNode
}) {
  const lang = await getRequestLang()
  const docs = loadRuntimeDocs(lang)
  const sections = getRuntimeSidebar('phpx', docs)

  return (
    <RuntimeDocsShell
      language="phpx"
      sections={sections}
      searchPlaceholder="Search PHPX runtime docs..."
    >
      {children}
    </RuntimeDocsShell>
  )
}
