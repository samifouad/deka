'use client'

import React, { createContext, useContext, useEffect, useState } from 'react'
import { Lang, LangEav } from '@/types'
import { languages } from '@/i18n'

type LangContext = {
    lang: Lang
    setLang: React.Dispatch<React.SetStateAction<Lang>>
}

const LangContext = createContext<LangContext | null>(null)

type LangContextProviderProps = {
    children: React.ReactNode
    initialLang: Lang
}

export function LangContextProvider({ children, initialLang }: LangContextProviderProps) {
    const [lang, setLang] = useState<Lang>(initialLang)
    const storageKey = 'deka-language'
    const cookieName = 'deka-language'

    const isValidLang = (value: string | null) =>
        Boolean(value && languages.some((item) => item.code === value))

    const readCookie = () => {
        if (typeof document === 'undefined') return null
        const cookies = document.cookie.split(';').map((item) => item.trim())
        const match = cookies.find((item) => item.startsWith(`${cookieName}=`))
        if (!match) return null
        return decodeURIComponent(match.split('=')[1] || '')
    }

    useEffect(() => {
        const cookieValue = readCookie()
        if (isValidLang(cookieValue)) {
            setLang(cookieValue as Lang)
            return
        }

        const saved = window.localStorage.getItem(storageKey)
        if (isValidLang(saved)) {
            setLang(saved as Lang)
        }
    }, [])

    useEffect(() => {
        window.localStorage.setItem(storageKey, lang)
        if (typeof document !== 'undefined') {
            document.cookie = `${cookieName}=${encodeURIComponent(lang)}; path=/; max-age=31536000`
        }
    }, [lang])

    return (
        <LangContext.Provider 
            value={{ 
                lang, 
                setLang 
            }}
        >
            {children}
        </LangContext.Provider>
    )
}

export function useLangContext() {
    const context = useContext(LangContext)
    if (!context) {
        throw new Error('useLangContext must be used within a LangContextProvider')
    }
    return context
}

export function useLangOnClient(): LangEav {
    const { lang, setLang } = useLangContext()
  
    if (lang === null) {
        return {data: null, action: null, error: {code: 404, message: 'unknown error with lang'}}
    }
  
    return {data: lang, action: setLang, error: null}
  }
