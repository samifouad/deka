/* eslint-disable @typescript-eslint/no-unused-vars */
import 'server-only'

//import { cookies } from 'next/headers'
import { ProfileEav } from '@/types'

export async function getProfile(slug?: string): Promise<ProfileEav> {

    // const sb = supabase()
    // const { data: sb_data, error: sb_error } = await sb.from('profiles').select('*')

    // sb.auth.stopAutoRefresh() // explicitly turning off this client-side feature

    // DEBUG
    //console.log('data: '+ JSON.stringify(data[0]))
    //console.log('error: '+ JSON.stringify(error))

    // if (sb_data === undefined)
    //     return {data: null, error: {code: 500, message: 'server error'}}

    // // if (sb_data && sb_data[0].username !== slug)
    // //     return {data: null, error: {code: 404, message: 'user not found'}}

    // if (sb_error)
     //   return {data: null, error: {code: 501, message: 'server error'}}

    return {data: null, error: null};
}