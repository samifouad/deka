import { useAuthOnClient } from "@/context/auth"
import { useThemeContext } from '@/context/theme'
import { update_theme } from '@/actions/update_theme'
import {
  SheetDescription,
  SheetHeader,
  SheetTitle
} from "@/components/ui/sheet"
import { Switch } from "@/components/ui/switch"

export default function LeftMenu () {

    const { data: auth } = useAuthOnClient()
    const { theme, setTheme } = useThemeContext()

    const handleThemeChange = async (newState: boolean) => {

        let newTheme: 'dark' | 'light'

        if (newState === true) {
            newTheme = 'dark'
        } else {
            newTheme = 'light'
        }

        setTheme(newTheme) // Update local theme state
        
        try {
            await update_theme(newTheme) // Update theme in the database
        } catch (error) {
            console.error('Failed to update theme:', error)
        }
    }

    return (
        <>
            {(auth && auth.setup_step && (auth.setup_step > 5)) && (
                <SheetHeader>
                    <SheetTitle>{auth ? '@'+ auth.username : 'edit your profile'}</SheetTitle>
                    <SheetDescription>
                        Make changes to your profile here. Click save when youre done.
                    </SheetDescription>
                </SheetHeader>
            )}
            <div className="table-cell py-4 align-middle">
                <form className="flex items-center space-x-2">
                    <span>dark mode</span>
                    <Switch
                    checked={theme === 'dark'}
                    onCheckedChange={handleThemeChange}
                    />
                </form>
                <br/><br/>
                <a href="/do/signout" className="text-sm text-red-500">sign out</a>
            </div>
        </>
    )
}