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
        <SidebarMenuButton @click="router.push('/dashboard')">
            <LayoutDashboard /> Dashboard
        </SidebarMenuButton>
        <SidebarMenuButton @click="router.push('/scan-tasks')">
            <ListTodo /> Scan Queue
        </SidebarMenuButton>
        <NavLibraries />
    </SidebarGroup>
</template>
