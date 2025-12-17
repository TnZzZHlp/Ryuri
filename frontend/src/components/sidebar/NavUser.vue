<script setup lang="ts">
import {
    IconDotsVertical,
    IconLogout,
    IconSettings,
} from "@tabler/icons-vue"

import {
    Avatar,
    AvatarFallback,
    AvatarImage,
} from "@/components/ui/avatar"
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuGroup,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
    useSidebar,
} from "@/components/ui/sidebar"
import { Dialog } from "@/components/ui/dialog"
import type { UserResponse } from "@/api/types"
import { useAuthStore } from "@/stores/useAuthStore"
import { useRouter } from "vue-router"
import { ref } from "vue"
import UserSettingsDialog from "./UserSettingsDialog.vue"

defineProps<{
    user: UserResponse
}>()

const { isMobile } = useSidebar()
const authStore = useAuthStore()
const router = useRouter()
const showSettings = ref(false)

function handleLogout() {
    authStore.logout()
    router.push('/login')
}

// Helper to get user initials for avatar fallback
function getUserInitials(username: string): string {
    return username.slice(0, 2).toUpperCase()
}
</script>

<template>
    <SidebarMenu>
        <SidebarMenuItem>
            <DropdownMenu>
                <DropdownMenuTrigger as-child>
                    <SidebarMenuButton size="lg"
                        class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground">
                        <Avatar class="h-8 w-8 rounded-lg grayscale">
                            <AvatarImage src="" :alt="user.username" />
                            <AvatarFallback class="rounded-lg">
                                {{ getUserInitials(user.username) }}
                            </AvatarFallback>
                        </Avatar>
                        <div class="grid flex-1 text-left text-sm leading-tight">
                            <span class="truncate font-medium">{{ user.username }}</span>
                            <span class="text-muted-foreground truncate text-xs">
                                ID: {{ user.id }}
                            </span>
                        </div>
                        <IconDotsVertical class="ml-auto size-4" />
                    </SidebarMenuButton>
                </DropdownMenuTrigger>
                <DropdownMenuContent class="w-(--reka-dropdown-menu-trigger-width) min-w-56 rounded-lg"
                    :side="isMobile ? 'bottom' : 'right'" :side-offset="4" align="end">
                    <DropdownMenuLabel class="p-0 font-normal">
                        <div class="flex items-center gap-2 px-1 py-1.5 text-left text-sm">
                            <Avatar class="h-8 w-8 rounded-lg">
                                <AvatarImage src="" :alt="user.username" />
                                <AvatarFallback class="rounded-lg">
                                    {{ getUserInitials(user.username) }}
                                </AvatarFallback>
                            </Avatar>
                            <div class="grid flex-1 text-left text-sm leading-tight">
                                <span class="truncate font-medium">{{ user.username }}</span>
                                <span class="text-muted-foreground truncate text-xs">
                                    ID: {{ user.id }}
                                </span>
                            </div>
                        </div>
                    </DropdownMenuLabel>
                    <DropdownMenuSeparator />
                    <DropdownMenuGroup>
                        <DropdownMenuItem @click="showSettings = true">
                            <IconSettings />
                            Settings
                        </DropdownMenuItem>
                    </DropdownMenuGroup>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem @click="handleLogout">
                        <IconLogout />
                        Logout
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
            <Dialog v-model:open="showSettings">
                <UserSettingsDialog :user="user" @close="showSettings = false" />
            </Dialog>
        </SidebarMenuItem>
    </SidebarMenu>
</template>
