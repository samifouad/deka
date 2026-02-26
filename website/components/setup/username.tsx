'use client'

import { useState } from 'react'
import { Card, CardContent } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { CheckCircle, Check } from 'lucide-react'

export default function Username() {
    const [username, setUsername] = useState('')
    const [isUsernameAvailable, setIsUsernameAvailable] = useState(false)
    const [step4Complete, setStep4Complete] = useState(false)

    const checkUsernameAvailability = (value: string) => {
        setUsername(value)
        setIsUsernameAvailable(value.length > 0)
    }

    const validateUsername = () => {
        setStep4Complete(isUsernameAvailable)
    }

    return (
        <Card className="relative flex-shrink-0 w-[90%] m-auto md:w-96 rounded-lg shadow-md bg-neutral-200 dark:bg-zinc-900">
          <CardContent>
            <div className="space-y-4">
              <div className="space-y-2">
                <div className="text-bold mt-8 mb-5 text-xl">@username</div>
                <div className="flex items-center">
                  <span className="bg-muted px-3 py-2 rounded-l-md border border-r-0 border-input">
                    conoda.co/
                  </span>
                  <div className="relative flex-grow">
                    <Input
                      id="username"
                      placeholder="username"
                      value={username}
                      onChange={(e) => checkUsernameAvailability(e.target.value)}
                      className="rounded-l-none"
                    />
                    {isUsernameAvailable && (
                      <Check className="absolute right-3 top-1/2 transform -translate-y-1/2 text-green-500" aria-hidden="true" />
                    )}
                  </div>
                </div>
              </div>
              <Button onClick={validateUsername} disabled={!isUsernameAvailable}>
                Set Username
              </Button>
            </div>
          </CardContent>
          {step4Complete && (
            <CheckCircle className="absolute top-4 right-4 text-green-500" aria-hidden="true" />
          )}
        </Card>
    )
}