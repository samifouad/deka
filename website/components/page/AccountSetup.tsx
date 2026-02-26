'use client'

import { useState, useRef } from 'react'
import { Button } from "@/components/ui/button"
import { SetupLang } from '@/components/setup/lang'
import { SetupTheme } from '@/components/setup/theme'
import Verify from '@/components/setup/verify'
import Username from '@/components/setup/username'
import Photo from '@/components/setup/photo'
import Finalize from '@/components/setup/finalize'

const steps = [
  { component: <SetupLang />, key: 'lang' },
  { component: <SetupTheme />, key: 'theme' },
  { component: <Verify />, key: 'verify' },
  { component: <Username />, key: 'username' },
  { component: <Photo />, key: 'photo' },
  { component: <Finalize />, key: 'finalize' },
]

export default function AccountSetup() {
  const [currentStep, setCurrentStep] = useState(0)
  const touchStartX = useRef<number | null>(null)

  const next = () => {
    setCurrentStep((prev) => Math.min(prev + 1, steps.length - 1))
  }

  const prev = () => {
    setCurrentStep((prev) => Math.max(prev - 1, 0))
  }

  const handleTouchStart = (e: React.TouchEvent) => {
    touchStartX.current = e.touches[0].clientX
  }

  const handleTouchEnd = (e: React.TouchEvent) => {
    if (touchStartX.current === null) return

    const deltaX = e.changedTouches[0].clientX - touchStartX.current

    if (deltaX > 50) {
      prev()
    } else if (deltaX < -50) {
      next()
    }

    touchStartX.current = null
  }

  const goToStep = (index: number) => {
    setCurrentStep(index)
  }

  return (
    <div className="relative container mx-auto p-0 overflow-hidden flex flex-col items-center">
      <h1 className="text-2xl font-bold mb-6 text-center">account setup</h1>

      {/* Card area */}
      <div 
        className="flex justify-center items-center relative w-full" 
        onTouchStart={handleTouchStart}
        onTouchEnd={handleTouchEnd}
      >
        {/* Left arrow */}
        <Button 
          onClick={prev}
          className="absolute left-0 top-1/2 transform -translate-y-1/2 z-10"
          variant="ghost"
          disabled={currentStep === 0}
        >
          ←
        </Button>

        {/* Center card */}
        <div className="relative w-[384px] h-[400px] overflow-hidden">
          <div
            className="flex transition-transform duration-500 ease-in-out"
            style={{ transform: `translateX(-${currentStep * 100}%)` }}
          >
            {steps.map((step, index) => (
              <div key={step.key} className="w-[384px] flex-shrink-0">
                {step.component}
              </div>
            ))}
          </div>
        </div>

        {/* Right arrow */}
        <Button 
          onClick={next}
          className="absolute right-0 top-1/2 transform -translate-y-1/2 z-10"
          variant="ghost"
          disabled={currentStep === steps.length - 1}
        >
          →
        </Button>
      </div>

      {/* Progress Dots */}
      <div className="flex space-x-2 mt-6">
        {steps.map((_, index) => {
          let color = 'bg-gray-400'
          if (index < currentStep) color = 'bg-green-500'   // Completed steps
          else if (index === currentStep) color = 'bg-blue-500' // Current step

          return (
            <button
              key={index}
              onClick={() => goToStep(index)}
              className={`w-4 h-4 rounded-full ${color} transition-colors duration-300`}
            />
          )
        })}
      </div>
    </div>
  )
}