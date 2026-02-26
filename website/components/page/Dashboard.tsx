'use client'

import React, { ReactNode, use } from 'react'
import MenuBar from '@/components/menu/Bar'
import { useThemeOnClient } from '@/context/theme'
import { useStepsOnClient } from '@/context/steps'


type DashboardProps = {
    children?: ReactNode
  };

export default function Dashboard ({children}: DashboardProps) {

    const { data: steps, error: steps_error } = useStepsOnClient()
    const { data: theme, error: theme_error } = useThemeOnClient()

    console.log('dashboard steps: ' + JSON.stringify(steps))

    return (
        <html lang="en" className={theme ?? 'light'}>
            <head>
                <title>conoda</title>
                <link rel="icon" href="/favicon.ico" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta name="description" content="from africa with love. commerce." />
                <link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png" />
                <link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png" />
                <link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png" />
            </head>
            <body>
                <div className="flex flex-col h-screen">
                    <div className="w-full h-[70px] border-none sticky">
                        <MenuBar user={'user'} />
                    </div>
                    <div className="flex-grow overflow-y-auto">
                        {children}
                    </div>
                </div>
            </body>
        </html>
    )
}