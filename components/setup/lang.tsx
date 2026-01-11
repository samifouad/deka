'use client'

import { useState } from 'react'
import { Card, CardContent } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { CheckCircle, Check, Upload, Loader2 } from 'lucide-react'
import { update_lang } from '@/actions/update_lang'
import { useStepsOnClient } from '@/context/steps'
import { useLangOnClient } from '@/context/lang'
import { Lang } from '@/types'
import { languages } from '@/i18n'

export function SetupLang() {
  const [loading, setLoading] = useState<string | null>(null) // Holds the code of the language being processed

  const { data: step, action: setStep } = useStepsOnClient()
  const { data: lang, action: setLang } = useLangOnClient()

  const handleLanguageChange = async (langCode: Lang) => {
    setLoading(langCode) // Set the language that is loading
    try {
      const { data, error } = await update_lang(langCode)

      if (data && setStep && setLang) {
        setLang(langCode) // Update the language in client context after successful server action
        setStep(1) // Move to the next step after successful server action
      } else {
        console.error("Failed to update language")
      }
    } catch (error) {
      console.error("An error occurred")
    } finally {
      setLoading(null) // Reset loading state
    }
  }

  return (
    <Card className="relative flex-shrink-0 w-[90%] m-auto my-0 md:w-96 rounded-lg shadow-md bg-neutral-200 dark:bg-zinc-900">
      <CardContent>
        <div className="space-y-4">
          <div className="space-y-2">
            <div className="text-bold mt-8 mb-5 text-xl">language</div>
            <div className="grid grid-cols-4 gap-2">
              {languages.map((langs) => (
                <Button
                  key={langs.code}
                  variant={lang === langs.code ? "default" : "outline"}
                  className="h-10 px-2 py-1 flex items-center justify-center"
                  onClick={() => handleLanguageChange(langs.code as Lang)}
                  disabled={loading !== null} // Disable all buttons during loading
                >
                  {loading === langs.code ? (
                    <Loader2 className="animate-spin w-4 h-4" /> // Spinner when loading
                  ) : (
                    langs.name
                  )}
                </Button>
              ))}
            </div>
          </div>
        </div>
      </CardContent>
      {step === 1 && !loading && (
        <CheckCircle className="absolute top-4 right-4 text-green-500" aria-hidden="true" />
      )}
      {loading && (
        <Loader2 className="absolute top-4 right-4 animate-spin w-6 h-6" />
      )}
    </Card>
  )
}