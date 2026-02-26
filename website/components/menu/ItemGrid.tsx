import IconGrid from '@/icons/grid'
import MenuItem from './Item'
import { MenuItemProps } from '@/types'

export default function MenuItemGrid ({ url, title, size }:MenuItemProps) {
    return (
        <MenuItem url={ url } title={ title }>
                <IconGrid size={ size ? size : '28px' } />
        </MenuItem>
    );
}