import { iconProps } from '@/types'
import { useThemeContext } from '@/context/theme'

/*
    name: mynaui:message
    from: https://icon-sets.iconify.design/mynaui/message/
*/

export default function IconsMessages ({ size }:iconProps) {

    const { theme } = useThemeContext()

    return (
            <svg xmlns="http://www.w3.org/2000/svg" width={ size } height={ size } viewBox="0 0 24 24">
                <path fill="none" stroke={theme === 'dark' ? '#FFF' : '#000'} strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.5" d="M3.464 16.828C2 15.657 2 14.771 2 11c0-3.771 0-5.657 1.464-6.828C4.93 3 7.286 3 12 3c4.714 0 7.071 0 8.535 1.172C22 5.343 22 7.229 22 11c0 3.771 0 4.657-1.465 5.828C19.072 18 16.714 18 12 18c-2.51 0-3.8 1.738-6 3v-3.212c-1.094-.163-1.899-.45-2.536-.96" />
            </svg>
    )
}