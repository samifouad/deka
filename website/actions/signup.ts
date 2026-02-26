'use server'

import { redirect } from 'next/navigation'
import { supabase } from '@/lib/supabase/server'

export async function signup(formData: FormData) {
    const sb = supabase()

    const formEmail = formData.get("email") as string
    const formPassword = formData.get("password") as string

    let redirect_url = '/signup'

    try {
          const { data, error } = await sb.auth.signUp({
            email: formEmail ?? "",
            password: formPassword ?? ""
          })

          if (data?.user && data.user.aud === "authenticated") {

            redirect_url = '/'

          } else {
            redirect_url = '/signup?error=user'
          }

          throw new Error('Not implemented')
        
    } catch (error) {

        redirect_url = '/signup?error=whoknows'
    }

    redirect(redirect_url)
  }