import 'server-only'

//import { cookies } from 'next/headers'
import { ProjectsEav } from '@/types'

export async function getProjects(): Promise<ProjectsEav> {

    // const sb = supabase()
    // const { data: project_data, error } = await sb.from('projects').select('*')

    // sb.auth.stopAutoRefresh() // explicitly turning off this client-side feature

    // if (project_data === undefined)
    //     return {data: null, error: {code: 500, message: 'server error'}}

    // if (error)
    //     return {data: null, error: {code: 500, message: 'server error'}}

    return {data: null, error: null};
}