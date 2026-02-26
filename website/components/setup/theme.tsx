'use client'

import { useState } from 'react'
import { Card, CardContent } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { CheckCircle } from 'lucide-react'

const themes = ['light', 'dark', 'system']

export function SetupTheme() {
    const [theme, setTheme] = useState('system')
    const [step2Complete, setStep2Complete] = useState(false)

    return (
        <Card className="relative flex-shrink-0 w-[90%] m-auto md:w-96 rounded-lg shadow-md bg-neutral-200 dark:bg-zinc-900">
          <CardContent>
            <div className="space-y-4">
              <div className="space-y-2">
                <div className="text-bold mt-8 mb-5 text-xl">theme</div>
                <div className="grid grid-cols-3 gap-2">
                  {themes.map((t) => (
                    <Button
                      key={t}
                      variant={theme === t ? "default" : "outline"}
                      className="h-10 px-2 py-1"
                      onClick={() => {
                        setTheme(t)
                        setStep2Complete(true)
                      }}
                    >
                      {t.charAt(0).toUpperCase() + t.slice(1)}
                    </Button>
                  ))}
                </div>
              </div>
            </div>
          </CardContent>
          {step2Complete && (
            <CheckCircle className="absolute top-4 right-4 text-green-500" aria-hidden="true" />
          )}
        </Card>
    )
}