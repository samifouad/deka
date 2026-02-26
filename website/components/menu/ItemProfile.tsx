/* eslint-disable @next/next/no-img-element */
import { Button } from "@/components/ui/button"

interface MenuItemProfileProps {
    url: string;
    user: string;
}

export default function MenuItemProfile ({ url, user }: MenuItemProfileProps) {

    return <img className="flex-item w-8 h-8 m-auto border-none rounded-full shadow-sm" src={url} alt={user} />;
}