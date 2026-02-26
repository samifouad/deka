import { iconProps } from '@/types'
import { useThemeContext } from '@/context/theme'

/*
    name: fluent:power-20-filled
    from: https://icon-sets.iconify.design/fluent/power-20-filled/
*/

export default function IconsBell ({ size }:iconProps) {

    const { theme } = useThemeContext()

    return (
        <svg xmlns="http://www.w3.org/2000/svg" width={ size } height={ size } viewBox="0 0 20 20">
            <path fill={theme === 'dark' ? '#FFF' : '#000'} d="M10.75 2.5a.75.75 0 0 0-1.5 0v6a.75.75 0 0 0 1.5 0zM13.743 4a.75.75 0 1 0-.748 1.3A6 6 0 1 1 7 5.305a.75.75 0 1 0-.75-1.3a7.5 7.5 0 1 0 7.493-.003" />
        </svg>
    )
}