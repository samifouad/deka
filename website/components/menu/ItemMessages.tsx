import IconMessages from '@/icons/messages'
import MenuItem from './Item'
import { MenuItemProps } from '@/types'

export default function MenuItemMessages ({ url, title, size }:MenuItemProps) {
    return (
        <MenuItem url={ url } title={ title }>
                <IconMessages size={ size ? size : '28px' } />
        </MenuItem>
    );
}