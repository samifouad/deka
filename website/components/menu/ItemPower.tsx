import IconPower from '@/icons/power'
import { MenuItemProps } from '@/types'

export default function MenuItemPower ({ url, title, size }:MenuItemProps) {
    return (
        <form action={ url } method="POST">
            <div className="h-14 flex-item min-w-14">
                <button title={ title } className="h-full w-full flex items-center justify-center pl-3 pr-3 rounded-lg hover:bg-gray-100">
                    <IconPower size={ size ? size : '28px' } />
                </button>
            </div>
        </form>
    );
}