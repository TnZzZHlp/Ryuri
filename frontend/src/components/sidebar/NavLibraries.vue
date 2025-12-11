<script setup lang="ts">
import { ChevronRight, Trash2, Settings, LibraryBig, MoreHorizontalIcon } from "lucide-vue-next"
import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
} from "@/components/ui/collapsible"
import {
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
    SidebarMenuSub,
    SidebarMenuSubItem,
} from "@/components/ui/sidebar"
import { Button } from "@/components/ui/button"
import { DropdownMenu, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'


defineProps<{
    libraries: {
        id: number
        title: string
        url: string
        isActive?: boolean
        items?: {
            title: string
            url: string
        }[]
    }[]
}>()

const handleSetting = (libraryId: number) => {
    console.log(libraryId)
}
</script>

<template>
    <SidebarMenu>
        <Collapsible as-child default-open class="group/collapsible">
            <SidebarMenuItem>
                <CollapsibleTrigger as-child>
                    <SidebarMenuButton tooltip="Libraries">
                        <component :is="LibraryBig" />
                        <span>Libraries</span>
                        <ChevronRight
                            class="ml-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90" />
                    </SidebarMenuButton>
                </CollapsibleTrigger>
                <CollapsibleContent>
                    <SidebarMenuSub>
                        <SidebarMenuSubItem v-for="library in libraries" :key="library.id">
                            <div class="flex items-center justify-between w-full">
                                <router-link :to="library.url" class="w-full hover:bg-sidebar-accent rounded-md">
                                    <span class="pl-2 font-extralight">{{ library.title }}</span>
                                </router-link>
                                <DropdownMenu>
                                    <DropdownMenuTrigger as-child>
                                        <Button variant="ghost" size="icon" class="w-6 h-6" aria-label="More Options"
                                            @click="() => handleSetting(library.id)">
                                            <MoreHorizontalIcon />
                                        </Button>
                                    </DropdownMenuTrigger>
                                    <DropdownMenuContent align="start">
                                        <DropdownMenuGroup>
                                            <DropdownMenuItem>
                                                <Settings />
                                                Settings
                                            </DropdownMenuItem>
                                            <DropdownMenuItem variant="destructive">
                                                <Trash2 />
                                                Delete
                                            </DropdownMenuItem>
                                        </DropdownMenuGroup>
                                    </DropdownMenuContent>
                                </DropdownMenu>
                            </div>
                        </SidebarMenuSubItem>
                    </SidebarMenuSub>
                </CollapsibleContent>
            </SidebarMenuItem>
        </Collapsible>
    </SidebarMenu>
</template>