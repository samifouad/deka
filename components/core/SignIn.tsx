'use client'

import { signin } from '@/actions/signin'
import { IconLogo } from '@/components/icons/logo'

export default function SignIn() {

  return (
      <div className="rounded-lg items-center shadow-lg bg-white bg-opacity-90 pt-0 pb-3 md:px-10 m-auto w-[90%] md:w-auto mt-[10vh] text-center">
          <div className="mb-3 flex justify-center items-center p-4">
            <IconLogo size='large' />
          </div>
          <form action={signin}>
              <input name="email" type="text" placeholder="email" className="shadow appearance-none border rounded w-[90%] md:w-[80%] py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"></input>
              <br/><br/>
              <input name="password" type="password" placeholder="password" className="shadow appearance-none border rounded w-[90%] md:w-[80%] py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"></input>
              <br /><br/>
              <button type="submit" className="bg-blue-800 hover:bg-blue-950 text-white font-bold py-2 px-4 rounded">Sign In</button>
              <br/><br/>
          </form>
        </div>
  )
}