import Link from 'next/link'

type Props = {
	title: string;
	body: string;
	href: string;
}

export default function Card ({title, body, href}: Props) {

    return (
        <div className="bg-white bg-opacity-90 p-7 border-none rounded-lg shadow-md shadow-black/10 shadow-lg/10 box-border">
            <Link href={href}>
                <h2>
                    {title}
                    <span>&rarr;</span>
                </h2>
                <p>
                    {body}
                </p>
            </Link>
        </div>
    )
}