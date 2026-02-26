import Link from 'next/link'
import { Icon } from '@iconify/react'

interface MenuItemProps {
    url: string;
    title: string;
    icon: string;
    iconSize: string;
}

export default function MenuItem ({ url, title, icon, iconSize }:MenuItemProps) {
    return (
        <div className="flex-item h-14">
            <Link href={url} title={title} className="h-full w-full flex items-center justify-center hover:bg-gray-100">
                <Icon icon={icon} style={{ fontSize: iconSize }} />
            </Link>
        </div>
    );
}