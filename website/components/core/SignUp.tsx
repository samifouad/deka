import { signup } from '@/actions/signup'

interface Props {
  redirect?: string
}

export default function SignUp({ redirect }: Props) {

  return (
      <div className="rounded-lg items-center shadow-lg bg-white bg-opacity-90 p-5 pt-20 mb-5 m-auto w-[90%] lg:w-[100%] h-50vh text-center">
          <form action={signup}>
              <input name="email" type="text" placeholder="email" className="shadow appearance-none border rounded w-[90%] md:w-[50%] py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"></input>
              <br/><br/>
              <input name="password" type="password" placeholder="password" className="shadow appearance-none border rounded w-[90%] md:w-[50%] py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"></input>
              <br /><br/>
              <button type="submit" className="bg-blue-800 hover:bg-blue-950 text-white font-bold py-2 px-4 rounded">Sign Up</button>
              <br/><br/>
          </form>
        </div>
  )
}