<script setup>
import { RouterView } from 'vue-router'
import SiteHeader from "@/components/header/SiteHeader.vue"
import {
    SidebarInset,
    SidebarProvider,
} from "@/components/ui/sidebar"
import AppSidebar from '@/components/sidebar/AppSidebar.vue'
import { KeepAlive } from 'vue';
import { useRoute } from 'vue-router';

const route = useRoute();
</script>

<template>
    <SidebarProvider :style="{
        '--sidebar-width': 'calc(var(--spacing) * 72)',
        '--header-height': 'calc(var(--spacing) * 12)',
    }">
        <AppSidebar variant="inset" />
        <SidebarInset>
            <SiteHeader />
            <div class="flex flex-1 flex-col max-h-screen overflow-scroll">
                <div class="@container/main flex flex-1 flex-col p-4">
                    <router-view v-slot="{ Component, route: currentRoute }">
                        <transition :name="currentRoute.meta.transition || 'fade'" mode="out-in">
                            <component :is="Component" :key="currentRoute.path" />
                        </transition>
                    </router-view>
                </div>
            </div>
        </SidebarInset>
    </SidebarProvider>
</template>