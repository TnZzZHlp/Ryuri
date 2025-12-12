<script setup lang="ts">
import { ChevronRight, Trash2, Settings, LibraryBig, LayoutDashboard } from "lucide-vue-next"
import {
    SidebarGroup,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
    SidebarMenuSub,
    SidebarMenuSubButton,
    SidebarMenuSubItem,
} from "@/components/ui/sidebar"
import NavLibraries from "./NavLibraries.vue"
import { useRouter } from "vue-router"

const router = useRouter()

</script>

<template>
    <SidebarGroup>
        <SidebarMenuButton @click="router.push('/dashboard')">
            <LayoutDashboard /> Dashboard
        </SidebarMenuButton>
        <NavLibraries />
    </SidebarGroup>
</template>
