<script setup>
import { RouterView } from 'vue-router'
import SiteHeader from "@/components/header/SiteHeader.vue"
import {
    SidebarInset,
    SidebarProvider,
} from "@/components/ui/sidebar"
import AppSidebar from '@/components/sidebar/AppSidebar.vue'
import { KeepAlive } from 'vue';
</script>

<template>
    <SidebarProvider :style="{
        '--sidebar-width': 'calc(var(--spacing) * 72)',
        '--header-height': 'calc(var(--spacing) * 12)',
    }">
        <AppSidebar variant="inset" />
        <SidebarInset>
            <SiteHeader />
            <div class="flex flex-1 flex-col">
                <div class="@container/main flex flex-1 flex-col gap-2">
                    <div class="flex flex-col gap-4 py-4 md:gap-6 md:py-6">
                        <router-view v-slot="{ Component }">
                            <keep-alive>
                                <component :is="Component" />
                            </keep-alive>
                        </router-view>
                    </div>
                </div>
            </div>
        </SidebarInset>
    </SidebarProvider>
</template>