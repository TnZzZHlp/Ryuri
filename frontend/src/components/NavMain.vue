<script setup lang="ts">
import { ChevronRight, Trash2, Settings, LibraryBig, MoreHorizontalIcon } from "lucide-vue-next"
import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
} from "@/components/ui/collapsible"
import {
    SidebarGroup,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
    SidebarMenuSub,
    SidebarMenuSubButton,
    SidebarMenuSubItem,
} from "@/components/ui/sidebar"
import { Button } from "@/components/ui/button"
import { DropdownMenu, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'


defineProps<{
    items: {
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

const handleSetting = (event: Event, libraryId: number) => {
    event.stopPropagation()
    event.preventDefault()
    console.log(libraryId)
}
</script>

<template>
    <SidebarGroup>
        <!-- <SidebarGroupLabel>Libraries</SidebarGroupLabel> -->
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
                            <SidebarMenuSubItem v-for="item in items" :key="item.id">
                                <div class="flex items-center justify-between w-full">
                                    <router-link :to="item.url">
                                        <span>{{ item.title }}</span>
                                    </router-link>
                                    <DropdownMenu>
                                        <DropdownMenuTrigger as-child>
                                            <Button variant="ghost" size="icon" class="w-6 h-6"
                                                aria-label="More Options">
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
    </SidebarGroup>
</template>
