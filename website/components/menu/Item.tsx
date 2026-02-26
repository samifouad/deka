import Link from 'next/link'
import { MenuItemProps } from '@/types'

export default function MenuItem ({ url, title, children }:MenuItemProps) {
    return (
        <div className="flex-item h-[60px] mt-[5px] min-w-[60px]">
            <Link href={ url } title={ title } className="h-full w-full flex items-center text-black dark:text-white pl-3 pr-3 justify-center rounded-lg hover:bg-accent">
                { children }
            </Link>
        </div>
    );
}