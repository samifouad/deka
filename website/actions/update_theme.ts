'use server'

import { Theme } from '@/types'

export async function update_theme(theme: Theme) {

    // const sb = supabase()

    try {
        throw new Error('Not implemented')
    //     const { data: profile_data, error: profile_error } = await sb.from('profiles').select('*').single()

    //     if (profile_error) {
    //         throw profile_error
    //     }

    //     sb.auth.stopAutoRefresh() // explicitly turning off this client-side feature

    //     const { data, error } = await sb
    //                                     .from('profiles')
    //                                     .update({ theme: theme })
    //                                     .eq('id', profile_data.id) // TODO: get from 


    //     if (error) {
    //         throw error
    //     }

    } catch (error) {
        throw error
    }
  }