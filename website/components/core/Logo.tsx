'use client'

/* eslint-disable @next/next/no-img-element */
import { useThemeContext } from '@/context/theme'

interface LogoProps {
  where?: string;
}

function Logo ({ where }: LogoProps) {
  
  const { theme } = useThemeContext()

  if (where == 'topMenu') {
    return (
      <div className="z-10 w-[40px] h-[40px]">
        <a href="/"><img src={'/logo-'+ theme +'.png'} alt="from africa with love" className="w-[40px] h-[40px]" /></a>
      </div>
    )
  }

    return (
      <div className="mb-[-1px] ml-[50px] z-10 w-[80px] h-[80px] rounded-[80px] shadow-md hover:shadow-xl">
				<a href="/"><img src="/logo.png" alt="from africa with love" className="w-[80px] h-[80px] rounded-[80px]" /></a>
		  </div>
    )
}

export default Logo;