'use client'

import React, { createContext, useContext, useEffect, useState } from 'react'
import type { Profile, AuthContext, AuthContextProviderProps, AuthEav } from '@/types'

const AuthContext = createContext<AuthContext | null>(null)

export function AuthContextProvider({ children, authState }: AuthContextProviderProps) {
    const [auth, setAuth] = useState<Profile>(authState)

    useEffect(() => {
        setAuth(authState);
      }, [authState]);

    return (
        <AuthContext.Provider value={{ auth, setAuth }}>
            {children}
        </AuthContext.Provider>
    )
}

// custom hook to make working with auth easier
export function useAuthContext() {
    const context = useContext(AuthContext)
    if (!context) {
        throw new Error('useAuthContext must be used within a AuthContextProvider')
    }
    return context
}

export function useAuthOnClient(): AuthEav {
    const { auth } = useAuthContext()

    if (auth === null) {
        return {data: null, error: {code: 404, message: 'User not found'}}
    }

    return {data: auth, error: null}
}