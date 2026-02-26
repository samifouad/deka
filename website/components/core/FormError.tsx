interface errorMsg {
    code: string
}

export default function FormError(props: errorMsg) {
    if (props.code !== "")
    {
      return (
          <div className="flex justify-center">
              <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded break-words relative w-3/4 mx-0 mt-9" role="alert">
                  { props.code ? <span className="text-xs">{ props.code }</span> : 'Please try again' }
              </div>
          </div>
      )
    } else {
      return (
        <></>
      )
    }
}