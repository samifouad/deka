import IconCreate from '@/icons/create'
import MenuItem from './Item'
import { MenuItemProps } from '@/types'

export default function MenuItemCreate ({ url, title, size }:MenuItemProps) {
    return (
        <MenuItem url={ url } title={ title }>
                <IconCreate size={ size ? size : '28px' } />
        </MenuItem>
    );
}