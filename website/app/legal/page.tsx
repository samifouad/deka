import Link from 'next/link'
import { Navbar } from '@/components/landing/Navbar'

export const metadata = {
  title: 'Legal | deka',
  description: 'Legal documents, open-source references, and contributor information for Deka.',
}

export default function LegalPage() {
  return (
    <div className="min-h-screen bg-background text-foreground">
      <Navbar />
      <main className="mx-auto max-w-4xl px-8 py-16 space-y-10">
        <header className="space-y-3">
          <h1 className="text-4xl font-bold text-foreground">legal</h1>
          <p className="text-lg text-muted-foreground">
            Terms, privacy, and open source acknowledgements for the Deka platform.
          </p>
        </header>

        <section className="space-y-4">
          <h2 className="text-xl font-semibold text-foreground">terms of use</h2>
          <p className="text-muted-foreground">
            Placeholder. Add your terms of use here when ready.
          </p>
          <Link href="#" className="text-sm text-primary hover:underline">
            terms of use (coming soon)
          </Link>
        </section>

        <section className="space-y-4">
          <h2 className="text-xl font-semibold text-foreground">privacy policy</h2>
          <p className="text-muted-foreground">
            Placeholder. Add your privacy policy here when ready.
          </p>
          <Link href="#" className="text-sm text-primary hover:underline">
            privacy policy (coming soon)
          </Link>
        </section>

        <section className="space-y-4">
          <h2 className="text-xl font-semibold text-foreground">open source references</h2>
          <p className="text-muted-foreground">
            Placeholder for open source contributor references and acknowledgements.
          </p>
        </section>
      </main>
    </div>
  )
}
