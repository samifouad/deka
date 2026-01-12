import "./globals.css"
import type { Metadata } from "next"
import { ClientLayout } from "@/components/ClientLayout"

export const metadata: Metadata = {
  title: "Deka - Self-hosted runtime for sovereign apps",
  description: "Run the Deka platform on your infrastructure with runtime services, a TypeScript framework, and deploy tooling built for production."
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  return (
    <html lang="en" className="light">
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
        <ClientLayout>
          {children}
        </ClientLayout>
      </body>
    </html>
  )
}
