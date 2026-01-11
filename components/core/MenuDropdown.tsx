'use client'

import * as React from "react"
import { Check, ChevronsUpDown } from "lucide-react"
import { useAuthContext } from '@/context/auth'
import { cn } from "@/lib/utils"
import { Button } from "@/components/ui/button"
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command"
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover"
import Link from "next/link"
import { useProjectsContext } from "@/context/projects"

export function MenuDropdown() {
  const [open, setOpen] = React.useState(false)
  const [value, setValue] = React.useState("")

  const { auth } = useAuthContext()
  const { projects } = useProjectsContext()
  let projectlist = projects

  console.log(value)

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="ghost"
          role="combobox"
          aria-expanded={open}
          className="w-[15px] outline-none"
        >
          <ChevronsUpDown className="ml-h-4 w-4 shrink-0 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[200px] p-0">
        <Command
          filter={(value, search) => {
            if (value.includes(search)) return 1
            return 0
          }}
        >
          <CommandInput placeholder="Search projects..."
          />
          <CommandList>
            <CommandEmpty>no projects found</CommandEmpty>
            <CommandGroup>
              {projectlist && projectlist.map((project) => (
                <CommandItem
                  key={project.project_id}
                  value={project.name}
                >
                  <Link href={'/'+ auth?.username +'/'+ project.slug}>{project.name}</Link>
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}