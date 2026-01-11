'use client'

import { useState } from 'react'
import { Card, CardContent } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { CheckCircle } from 'lucide-react'

export default function Verify() {
    const [step3Complete, setStep3Complete] = useState(false)
    const [emailCode, setEmailCode] = useState('')

    const validateEmailCode = () => {
        const isValid = emailCode.length === 6
        setStep3Complete(isValid)
    }

    const resendEmailCode = () => {
        console.log('Resending email code')
    }

    return (
        <Card className="relative flex-shrink-0 w-[90%] m-auto md:w-96 rounded-lg shadow-md bg-neutral-200 dark:bg-zinc-900">
          <CardContent>
            <div className="space-y-4">
              <div className="space-y-2">
                <div className="text-bold mt-8 mb-5 text-xl">verify</div>
                <Input
                  id="email-code"
                  placeholder="Enter 6-digit code"
                  value={emailCode}
                  onChange={(e) => setEmailCode(e.target.value)}
                  maxLength={6}
                />
              </div>
              <div className="flex space-x-2">
                <Button onClick={validateEmailCode}>Validate Code</Button>
                <Button variant="outline" onClick={resendEmailCode}>Resend Code</Button>
              </div>
            </div>
          </CardContent>
          {step3Complete && (
            <CheckCircle className="absolute top-4 right-4 text-green-500" aria-hidden="true" />
          )}
        </Card>
    )
}