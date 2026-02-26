'use client'

import Logo from '@/components/core/Logo'
import Recovery from '@/components/core/Recovery'
import Links from '@/components/core/Links'

// Skip static generation - uses theme context
export const dynamic = 'force-dynamic'

export default function Recover () {
    return (
        <>
            <Logo />
            <Recovery />
            <Links target='main' />
        </>
    )
}