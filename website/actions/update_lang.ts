'use server'

import { Lang, LangEav } from '@/types'
import { supabase } from '@/lib/supabase/server'

export async function update_lang(lang: Lang): Promise<LangEav> {

        //console.log('trying to set lang to: ' + lang)

        const sb = supabase()

        const { data: profile_data, error: profile_error } = await sb.from('profiles').select('*').single()

        if (profile_error) {
            return {data: null, action: null, error: {code: 500, message: 'server error'}}
        }

        sb.auth.stopAutoRefresh() // explicitly turning off this client-side feature

        const { data: sb_data, error: sb_error } = await sb
                                        .from('profiles')
                                        .update({ lang: lang })
                                        .eq('id', profile_data.id) // TODO: get from 
                                        .select('lang')
                                        .single()

        if (sb_data === undefined)
            return {data: null, action: null, error: {code: 500, message: 'server error'}}

        //console.log('returning: ' + JSON.stringify(sb_data))

        return {data: sb_data?.lang, action: null, error: null};

        throw new Error('Not implemented')
  }