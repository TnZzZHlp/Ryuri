<script setup lang="ts">
import { IconInnerShadowTop } from "@tabler/icons-vue"
import NavMain from "./NavMain.vue";
import NavUser from "@/components/NavUser.vue"
import {
    Sidebar,
    SidebarContent,
    SidebarFooter,
    SidebarHeader,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
} from "@/components/ui/sidebar"
import { computed, onBeforeMount } from "vue";
import { useAuthStore } from "@/stores/useAuthStore";
import { useLibraryStore } from "@/stores/useLibraryStore";

const authStore = useAuthStore()
const libraryStore = useLibraryStore()

const libraries = computed(() => libraryStore.libraries.map((lib) => {
    return {
        id: lib.id,
        title: lib.name,
        url: `/library/${lib.id}`,
    }
}))

onBeforeMount(() => {
    // fetch user information if authenticated but user not loaded
    if (authStore.isAuthenticated && !authStore.user) {
        authStore.fetchUser()
    }

    // fetch library information if authenticated but library not loaded
    if (libraryStore.libraries.length == 0) {
        libraryStore.fetchLibraries()
    }
})
</script>

<template>
    <Sidebar collapsible="offcanvas">
        <SidebarHeader>
            <SidebarMenu>
                <SidebarMenuItem>
                    <SidebarMenuButton as-child class="data-[slot=sidebar-menu-button]:!p-1.5">
                        <router-link to="/">
                            <IconInnerShadowTop class="!size-5" />
                            <span class="text-base font-semibold">Wyuri</span>
                        </router-link>
                    </SidebarMenuButton>
                </SidebarMenuItem>
            </SidebarMenu>
        </SidebarHeader>
        <SidebarContent>
            <NavMain :items="libraries" />
        </SidebarContent>
        <SidebarFooter>
            <NavUser v-if="authStore.user" :user="authStore.user" />
        </SidebarFooter>
    </Sidebar>
</template>
