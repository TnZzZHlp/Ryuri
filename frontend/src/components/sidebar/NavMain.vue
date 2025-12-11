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

</script>

<template>
    <SidebarGroup>
        <SidebarMenuButton @click="router.push('/dashboard')">
            <LayoutDashboard /> Dashboard
        </SidebarMenuButton>
        <NavLibraries :libraries="libraries" />
    </SidebarGroup>
</template>
