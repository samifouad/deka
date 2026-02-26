/* eslint-disable @next/next/no-img-element */
'use client' 

import { usePathname } from 'next/navigation'
import Link from 'next/link'
import Logo from '@/components/core/Logo'
import MenuItemProfile from '@/components/menu/ItemProfile'
import MenuItemBell from '@/components/menu/ItemBell'
import MenuItemMessages from '@/components/menu/ItemMessages'
import LeftMenu from '@/components/page/LeftMenu'
import { Button } from "@/components/ui/button"
import { Sheet, SheetContent, SheetTrigger } from "@/components/ui/sheet"
import { MenuDropdown } from '@/components/core/MenuDropdown'
import { IconLogo } from '@/components/icons/logo'
import { useAuthOnClient } from '@/context/auth'

export default function MenuBar({user}: any) {

    const pathname = usePathname()
    const segments = pathname?.split('/').filter(Boolean)
    const page = segments[0]
    const section = segments[1]

    // const core = ['messages', 'notifications', 'create', 'profile', 'settings']
    // const isCore = core.includes(page)

    const { data: auth, error } = useAuthOnClient()

    return (
        <div className="flex flex-row justify-between h-[100%]">
            <div className="flex items-center">
              <div className="ml-5">
                {/* <Logo where="topMenu" /> */}
                <IconLogo size='small' />
              </div>
              {(auth && (auth.username !== '') && auth.setup_step && (auth.setup_step > 4)) ? (
                <>
                  <div className="ml-5 text-3xl text-slate-200">
                    /
                  </div>
                  <div className="flex ml-2 items-center h-full font-bold">
                    <Link href={'/'+ user} className="h-[80%] flex items-center rounded-lg hover:bg-gray-100 dark:hover:bg-accent px-2">
                        @{user}
                    </Link>
                    <MenuDropdown />
                  </div>
                </>
              ): null}
              {(section && auth && (auth.username !== '') && auth.setup_step && (auth.setup_step > 4)) ? (
                <>
                  <div className="ml-3 text-3xl text-slate-200">
                    /
                  </div>
                  <div className="ml-4 font-bold">
                    <Link href={'/'+ user + '/' + section}>{section}</Link>
                  </div>
                </>
              ): null}
            </div>



            <div className="flex flex-row justify-between mr-5">
                {(auth && (auth.username !== '') && auth.setup_step && (auth.setup_step > 4)) ? (
                  <>
                    <MenuItemMessages url="/messages" title="Messages" size='24px' />
                    <MenuItemBell url="/notifications" title="Notifications" size='20px' />
 
                    <Sheet>
                        <SheetTrigger asChild>
                            <div className="h-[60px] mt-[5px] mb-[3px] flex-item bg-none dark:bg-none">
                                <Button variant="ghost" title="Menu" className="h-full w-full flex flex-col items-center justify-center pl-3 pr-3 rounded-lg bg-none dark:bg-none hover:bg-accent">
                                    <MenuItemProfile url="https://images.unsplash.com/photo-1518288774672-b94e808873ff?q=80&w=1938&auto=format&fit=crop&ixlib=rb-4.0.3&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D" user="sami" />
                                </Button>
                            </div>
                        </SheetTrigger>
                        <SheetContent>
                            <LeftMenu />
                        </SheetContent>
                    </Sheet>
                  </>
                ): null}
            </div>
        </div>
    )
}