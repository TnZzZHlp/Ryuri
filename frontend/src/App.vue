<script setup lang="ts">
import { RouterView } from 'vue-router';
import 'vue-sonner/style.css'
import { Toaster } from '@/components/ui/sonner'
</script>

<template>
    <RouterView v-slot="{ Component, route }">
        <transition name="fade" mode="out-in">
            <component :is="Component" :key="route.matched[0]?.path ?? route.path" />
        </transition>
    </RouterView>
    <Toaster position="top-center" rich-colors />
</template>
