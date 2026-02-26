import { iconProps } from '@/types'
import { useThemeContext } from '@/context/theme'

/*
    name: gala:add
    from: https://icon-sets.iconify.design/gala/add/
*/

export default function IconsAdd ({ size }:iconProps) {

    const { theme } = useThemeContext()

    return (
        <svg xmlns="http://www.w3.org/2000/svg" width={ size } height={ size } viewBox="0 0 256 256">
            <g fill="none" stroke={theme === 'dark' ? '#FFF' : '#000'} stroke-linecap="round" stroke-linejoin="round" stroke-width="16">
                <circle cx="128" cy="128" r="112" />
                <path d="M 79.999992,128 H 176.0001" />
                <path d="m 128.00004,79.99995 v 96.0001" />
            </g>
        </svg>
    )
}