'use server'

import { supabase, supabase_service } from '@/lib/supabase/server'
import { redirect } from 'next/navigation'
import { customAlphabet } from 'nanoid'

export async function add_device(formData: FormData) {

    const sb = supabase_service()

     // Extract the device code from the form data
    const device_code = formData.get('device_code');

    // Log the device code for debugging
    console.log('Received device code:', device_code);

    try {
        // match code to stored device authorization request
        const { data, error } = await sb.from('device_auth').select('*').eq('user_code', device_code).single()

        console.log('Received data:', data);

        if (error)
            throw (error)

        try {
            // generate token
            const nanoid = customAlphabet('AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtVuVvWwXxYyZz123467890', 24)
            const generated_token = nanoid();   

            const sbc = supabase()

            // place token inside devices table to trigger a successful authentication in cli
            const { data: tdata, error: terror } = await sbc.from('devices').insert({
                device_id: data.device_id,
                client_version: data.client_version,
                machine_type: data.machine_type,
                os_version: data.os_version,
                machine_arch: data.machine_arch,
                ip_fingerprint: data.ip_fingerprint,
                token: generated_token
            })

            if (terror) {
                console.log(terror)
                throw (terror)
            }

        } catch (e) {
            throw(e)
        }
        
    } catch (error) {
        redirect('/?error=whoknows')
    }

    redirect('/?success')
  }