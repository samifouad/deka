import Link from 'next/link'
import { getRuntimeSidebar } from '@/lib/runtime-docs'
import { loadRuntimeDocs } from '@/lib/runtime-docs-server'
import { getRequestLang } from '@/lib/i18n-server'

export default async function PHPRuntimeHomePage() {
  const lang = await getRequestLang()
  const docs = loadRuntimeDocs(lang)
  const sections = getRuntimeSidebar('php', docs)

  return (
    <div className="max-w-5xl mx-auto px-8 py-12">
      <div className="space-y-6">
        <div>
          <h1 className="text-4xl font-bold text-foreground mb-2">
            PHP runtime reference
          </h1>
          <p className="text-xl text-muted-foreground">
            API reference and runtime behavior for Deka PHP handlers, modules, and configuration.
          </p>
        </div>

        <div className="border-l-4 border-primary pl-4 py-2">
          <p className="text-muted-foreground">
            This section will be generated from code comments. Use the sidebar to browse the API
            surfaces and runtime primitives.
          </p>
        </div>

        <div className="grid md:grid-cols-2 gap-6 pt-4">
          {sections.map((section) => (
            <div key={section.categorySlug} className="border border-border rounded-lg p-6">
              <h3 className="text-lg font-semibold text-foreground mb-2">
                {section.category}
              </h3>
              <ul className="space-y-2 text-sm text-muted-foreground">
                {section.items.map((item) => (
                  <li key={item.slug}>
                    <Link
                      href={`/docs/php/${item.slug}`}
                      className="text-foreground hover:text-primary"
                    >
                      {item.name}
                    </Link>
                    <div className="text-xs text-muted-foreground">{item.description}</div>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
