<script setup lang="ts">
import { LayoutDashboard, ListTodo } from "lucide-vue-next"
import {
    SidebarGroup,
    SidebarMenuButton,
} from "@/components/ui/sidebar"
import NavLibraries from "./NavLibraries.vue"
import { useRouter } from "vue-router"
import { useI18n } from "vue-i18n"

const router = useRouter()
const { t } = useI18n()

</script>

<template>
    <SidebarGroup>
        <SidebarMenuButton @click="router.push({ name: 'Dashboard' })"
            :is-active="router.currentRoute.value.name === 'Dashboard'">
            <LayoutDashboard /> {{ t('nav.dashboard') }}
        </SidebarMenuButton>
        <SidebarMenuButton @click="router.push({ name: 'ScanTasks' })"
            :is-active="router.currentRoute.value.name === 'ScanTasks'">
            <ListTodo /> {{ t('nav.scan_queue') }}
        </SidebarMenuButton>
        <NavLibraries />
    </SidebarGroup>
</template>
