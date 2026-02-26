'use client'

import React, { createContext, useContext, useEffect, useState } from 'react'
import type { Theme, ThemeContext, ThemeContextProviderProps, ThemeEav } from '@/types'

const ThemeContext = createContext<ThemeContext | null>(null)

export function ThemeContextProvider({ children, initialTheme }: ThemeContextProviderProps) {
    const [theme, setTheme] = useState<Theme>(initialTheme)

    useEffect(() => {
        if (theme === 'dark') {
          document.documentElement.classList.add('dark')
        } else {
          document.documentElement.classList.remove('dark')
        }
      }, [theme])

    return (
        <ThemeContext.Provider value={{ theme, setTheme }}>
            {children}
        </ThemeContext.Provider>
    )
}

// custom hook to make working with theme easier
export function useThemeContext() {
    const context = useContext(ThemeContext)
    if (!context) {
        // During SSR/pre-rendering, return default theme
        // This allows pages to be pre-rendered without context
        return { theme: 'dark' as Theme, setTheme: () => {} }
    }
    return context
}

export function useThemeOnClient(): ThemeEav {
  const { theme } = useThemeContext()

  if (theme === null) {
      return {data: null, error: {code: 404, message: 'User not found'}}
  }

  return {data: theme, error: null}
}