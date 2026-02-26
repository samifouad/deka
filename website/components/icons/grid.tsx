import { iconProps } from '@/types'
import { useThemeContext } from '@/context/theme'

/*
    name: fluent:grid-dots-20-filled
    from: https://icon-sets.iconify.design/fluent/grid-dots-20-filled/
*/

export default function IconsGrid ({ size }:iconProps) {

    const { theme } = useThemeContext()

    return (
        <svg xmlns="http://www.w3.org/2000/svg" width={ size } height={ size } viewBox="0 0 20 20">
            <path fill={theme === 'dark' ? '#FFF' : '#000'} d="M5.75 4a1.75 1.75 0 1 1-3.5 0a1.75 1.75 0 0 1 3.5 0m0 6a1.75 1.75 0 1 1-3.5 0a1.75 1.75 0 0 1 3.5 0M4 17.75a1.75 1.75 0 1 0 0-3.5a1.75 1.75 0 0 0 0 3.5M11.75 4a1.75 1.75 0 1 1-3.5 0a1.75 1.75 0 0 1 3.5 0M10 11.75a1.75 1.75 0 1 0 0-3.5a1.75 1.75 0 0 0 0 3.5M11.75 16a1.75 1.75 0 1 1-3.5 0a1.75 1.75 0 0 1 3.5 0M16 5.75a1.75 1.75 0 1 0 0-3.5a1.75 1.75 0 0 0 0 3.5M17.75 10a1.75 1.75 0 1 1-3.5 0a1.75 1.75 0 0 1 3.5 0M16 17.75a1.75 1.75 0 1 0 0-3.5a1.75 1.75 0 0 0 0 3.5" />
        </svg>
    )
}