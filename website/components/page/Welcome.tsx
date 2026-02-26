'use client'

import React, { ReactNode } from 'react'
import '../../app/globals.css'

type Props = {
    children?: ReactNode; // Explicitly type children prop
}

export default function Welcome({children}: Props) {
    return (
        <html lang="en">
            <head>
                <title>conoda</title>
                <link rel="icon" href="/favicon.ico" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta name="description" content="conoda. build and deploy." />
                <link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png" />
                <link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png" />
                <link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png" />
                <link rel="preload" as="image" href="/logo.png" />
            </head>
            <body className="bg-[#EEE9DF]">
                <div className="h-screen w-[100%] m-auto">
                    <main className="w-[98%] md:w-[100%] m-auto md:m-0">
                        <section className="w-full m-auto mt-[5%]">
                            {children}
                        </section>
                    </main> 
                </div>
            </body>
        </html>
    )
} 