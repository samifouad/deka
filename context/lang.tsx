'use client'

import React, { createContext, useContext, useState } from 'react'
import { Lang, LangEav } from '@/types'

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