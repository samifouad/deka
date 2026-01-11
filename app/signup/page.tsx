import Logo from '@/components/core/Logo'
import SignUp from '@/components/core/SignUp'
import Links from '@/components/core/Links'

interface Props {
    searchParams: Promise<Record<string, string>>
}

export default async function SignUpPage(props: Props) {
    const searchParams = await props.searchParams;

    if (searchParams.r === null || searchParams.r === undefined) {
        return (
            <>
                <Logo />
                <SignUp />
                <Links target='signin' />            
            </>
        )
    } else {
        return (
            <>
                <Logo />
                <SignUp redirect={searchParams.r} />
                <Links target='signin' />            
            </>
        )
    }
}