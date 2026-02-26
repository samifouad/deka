'use server'

import { cookies } from 'next/headers'
import { NextRequest, NextResponse } from 'next/server'
import { redirect } from 'next/navigation'

export async function signin(formData: FormData) {

    const formEmail = formData.get("email") as string
    const formPassword = formData.get("password") as string

    try {
        // const { data, error } = await sb.auth.signInWithPassword({
        //   email: formEmail ?? "",
        //   password: formPassword ?? ""
        // })

        // if (data.user && data.user.aud === "authenticated") {
        //   console.log('user is authenticated!')
        // } else {
        //   redirect('/?error=user')
        // }

        throw new Error('Not implemented')
        
    } catch (error) {
        console.log(error)
        redirect('/?error=whoknows')
    }

    redirect('/')
  }