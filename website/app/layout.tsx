import "./globals.css"
import type { Metadata } from "next"
import { ClientLayout } from "@/components/ClientLayout"
import { getRequestLang } from "@/lib/i18n-server"

export const metadata: Metadata = {
  title: "Deka - Self-hosted runtime for sovereign apps",
  description: "Run the Deka platform on your infrastructure with runtime services, a TypeScript framework, and deploy tooling built for production.",
  manifest: "/site.webmanifest",
  icons: {
    icon: [
      { url: "/favicon-16x16.png", sizes: "16x16", type: "image/png" },
      { url: "/favicon-32x32.png", sizes: "32x32", type: "image/png" },
    ],
    apple: [{ url: "/apple-touch-icon.png", sizes: "180x180", type: "image/png" }],
  },
}

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  const initialLang = await getRequestLang()

  return (
    <html lang={initialLang} className="light">
      <head>
        <style>{`
          .search-panel {
            opacity: 0;
            pointer-events: none;
            transform: translateY(0);
          }

          .search-active .search-panel {
            pointer-events: auto;
          }

          .search-open .search-panel {
            opacity: 1;
          }
        `}</style>
      </head>
      <body className="antialiased">
        <ClientLayout initialLang={initialLang}>
          {children}
        </ClientLayout>
      </body>
    </html>
  )
}
