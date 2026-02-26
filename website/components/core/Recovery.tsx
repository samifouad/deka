'use client'

export default function Recover () {

    return (
            <div className="rounded-lg items-center shadow-lg bg-white bg-opacity-90 p-5 pt-20 mb-5 m-auto w-[90%] lg:w-[100%] h-50vh text-center">
                <form>
                    <input className="shadow appearance-none border rounded w-[90%] md:w-[50%] py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline" id="email" type="text" placeholder="coming soon" value=""></input>
                    <br/><br/>
                    <button 
                        className="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded disabled:opacity-50 disabled:cursor-not-allowed" 
                        type="submit" 
                        disabled={true}
                    >
                        Recover
                    </button>
                    <br/><br/>
                </form>
            </div>
    )
}