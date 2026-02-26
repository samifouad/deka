import Card from '@/components/core/Card'

interface page {
    target: string
}

export default function Links (props: page) {
    if (props.target === "signup") {
        return (
            <div className="w-full lg:w-[100%] mb-5">
                <div className="w-[80%] m-auto md:w-[44%] mb-4 md:float-left">
                    <Card
                        href="/recovery"
                        title="Recovery"
                        body="Lost password?"
                    />
                </div>
                <div className="w-[80%] m-auto md:w-[44%] mb-4 md:float-right">
                    <Card
                        href="/signup"
                        title="Sign Up"
                        body="It's fast &amp; free."
                    />
                </div>
            </div>
        )
    } else if (props.target === "signin") {
        return (
            <div className="w-full lg:w-[100%] mb-5">
                <div className="w-[80%] m-auto md:w-[44%] mb-4 md:float-left">
                    <Card
                        href="/recovery"
                        title="Recovery"
                        body="Lost password?"
                    />
                </div>
                <div className="w-[80%] m-auto md:w-[44%] mb-4 md:float-right">
                    <Card
                        href="/"
                        title="Sign In"
                        body="All aboard."
                    />
                </div>
            </div>
        )
    } else {
        return (
            <div className="w-full lg:w-[100%] mb-5">
                <div className="w-[80%] m-auto md:w-[44%] mb-4 md:float-left">
                    <Card
                        href="/"
                        title="Sign In"
                        body="All aboard."
                    />
                </div>
                <div className="w-[80%] m-auto md:w-[44%] mb-4 md:float-right">
                    <Card
                        href="/signup"
                        title="Sign Up"
                        body="It's fast &amp; free."
                    />
                </div>
            </div>
        )
    }
}