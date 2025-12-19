<script setup lang="ts">
import NavMain from "./NavMain.vue";
import NavUser from "@/components/sidebar/NavUser.vue"
import {
    Sidebar,
    SidebarContent,
    SidebarFooter,
    SidebarHeader,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
} from "@/components/ui/sidebar"
import { onBeforeMount } from "vue";
import { useAuthStore } from "@/stores/useAuthStore";

const authStore = useAuthStore()

onBeforeMount(() => {
    // fetch user information if authenticated but user not loaded
    if (authStore.isAuthenticated && !authStore.user) {
        authStore.fetchUser()
    }
})
</script>

<template>
    <Sidebar collapsible="offcanvas">
        <SidebarHeader>
            <SidebarMenu>
                <SidebarMenuItem>
                    <SidebarMenuButton as-child class="data-[slot=sidebar-menu-button]:p-1.5!">
                        <router-link to="/">
                            <img src="/ryuri.svg" class="size-5!" alt="Ryuri Logo" />
                            <span class="text-base font-semibold">Ryuri</span>
                        </router-link>
                    </SidebarMenuButton>
                </SidebarMenuItem>
            </SidebarMenu>
        </SidebarHeader>
        <SidebarContent>
            <NavMain />
        </SidebarContent>
        <SidebarFooter>
            <NavUser v-if="authStore.user" :user="authStore.user" />
        </SidebarFooter>
    </Sidebar>
</template>
