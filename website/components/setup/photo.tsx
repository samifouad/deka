'use client'

import { useState } from 'react'
import { Card, CardContent } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { CheckCircle, Upload, Check } from 'lucide-react'

export default function Photo() {
    const [profilePicture, setProfilePicture] = useState<File | null>(null)
    const [step4Complete, setStep4Complete] = useState(false)

    const handleProfilePictureChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        if (e.target.files && e.target.files[0]) {
          setProfilePicture(e.target.files[0])
          setStep4Complete(true)
        }
    }

    return (
        <Card className="relative flex-shrink-0 w-[90%] m-auto md:w-96 rounded-lg shadow-md bg-neutral-200 dark:bg-zinc-900">
          <CardContent>
            <div className="space-y-4">
              <div className="space-y-2">
                <div className="text-bold mt-8 mb-5 text-xl">photo</div>
                <div className="flex items-center justify-center w-full">
                  <label
                    htmlFor="profile-picture"
                    className="flex flex-col items-center justify-center w-full h-64 border-2 border-dashed rounded-lg cursor-pointer bg-gray-50 dark:hover:bg-bray-800 dark:bg-gray-700 hover:bg-gray-100 dark:border-gray-600 dark:hover:border-gray-500 dark:hover:bg-gray-600"
                  >
                    <div className="flex flex-col items-center justify-center pt-5 pb-6">
                      <Upload className="w-8 h-8 mb-4 text-gray-500 dark:text-gray-400" />
                      <p className="mb-2 text-sm text-gray-500 dark:text-gray-400">
                        <span className="font-semibold">Click to upload</span> or drag and drop
                      </p>
                      <p className="text-xs text-gray-500 dark:text-gray-400">
                        SVG, PNG, JPG or GIF (MAX. 800x400px)
                      </p>
                    </div>
                    <Input
                      id="profile-picture"
                      type="file"
                      className="hidden"
                      onChange={handleProfilePictureChange}
                      accept="image/*"
                    />
                  </label>
                </div>
              </div>
              {profilePicture && (
                <p className="text-sm text-green-500">
                  File selected: {profilePicture.name}
                </p>
              )}
            </div>
          </CardContent>
          {step4Complete && (
            <CheckCircle className="absolute top-4 right-4 text-green-500" aria-hidden="true" />
          )}
        </Card>
    )
}