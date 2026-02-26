import IconBell from '@/icons/bell'
import MenuItem from './Item'
import { MenuItemProps } from '@/types'

export default function MenuItemBell ({ url, title, size }:MenuItemProps) {
    return (
        <MenuItem url={ url } title={ title }>
                <IconBell size={ size ? size : '28px' } />
        </MenuItem>
    );
}