'use client'

import { useState } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { Label } from "@/components/ui/label"

export default function Finalize() {

    const [step5Complete, setStep5Complete] = useState(false)
    const [termsAgreed, setTermsAgreed] = useState(false)

    const completeSetup = () => {
        if (termsAgreed) {
        console.log('Setup complete')
        }
    }

    return (
        <Card className="relative flex-shrink-0 w-[90%] m-auto md:w-96 rounded-lg shadow-md bg-neutral-200 dark:bg-zinc-900">
          <CardContent>
            <div className="space-y-4">
                <div className="text-bold mt-8 mb-5 text-xl">complete</div>
              <div className="flex items-center space-x-2">
                <Checkbox
                  id="terms"
                  checked={termsAgreed}
                  onCheckedChange={(checked) => setTermsAgreed(checked as boolean)}
                />
                <Label htmlFor="terms" className="text-sm">
                  I agree to the <a href="#" className="text-blue-500 hover:underline">Terms of Service</a>
                </Label>
              </div>
              <Button onClick={completeSetup} disabled={!termsAgreed}>
                Complete Setup
              </Button>
            </div>
          </CardContent>
        </Card>
    )
}