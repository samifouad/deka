'use client'

import React, { createContext, useContext, useEffect, useState } from 'react'
import type { Projects, ProjectsContext, ProjectsContextProviderProps } from '@/types'

const ProjectsContext = createContext<ProjectsContext | null>(null)

export function ProjectsContextProvider({ children, projectsState }: ProjectsContextProviderProps) {
    const [projects, setProjects] = useState<Projects>(projectsState)

    return (
        <ProjectsContext.Provider value={{ projects, setProjects }}>
            {children}
        </ProjectsContext.Provider>
    )
}

// custom hook to make working with projects easier
export function useProjectsContext() {
    const context = useContext(ProjectsContext)
    if (!context) {
        throw new Error('useProjectsContext must be used within a ProjectsContextProvider')
    }
    return context
}