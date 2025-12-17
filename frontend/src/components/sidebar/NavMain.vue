<script setup lang="ts">
import { LayoutDashboard, ListTodo } from "lucide-vue-next"
import {
    SidebarGroup,
    SidebarMenuButton,
} from "@/components/ui/sidebar"
import NavLibraries from "./NavLibraries.vue"
import { useRouter } from "vue-router"

const router = useRouter()

</script>

<template>
    <SidebarGroup>
        <SidebarMenuButton @click="router.push({ name: 'Dashboard' })"
            :is-active="router.currentRoute.value.name === 'Dashboard'">
            <LayoutDashboard /> Dashboard
        </SidebarMenuButton>
        <SidebarMenuButton @click="router.push({ name: 'ScanTasks' })"
            :is-active="router.currentRoute.value.name === 'ScanTasks'">
            <ListTodo /> Scan Queue
        </SidebarMenuButton>
        <NavLibraries />
    </SidebarGroup>
</template>
