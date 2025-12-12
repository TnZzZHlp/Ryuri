<script setup lang="ts">
import { SidebarTrigger } from "@/components/ui/sidebar"
import { Button } from "@/components/ui/button"
import { Sun, ArrowLeft } from "lucide-vue-next";
import { computed } from "vue";
import { useRouter } from "vue-router";

const router = useRouter();

// change app theme
const changeAppTheme = () => {
    const html = document.querySelector("html")
    if (html) {
        html.classList.toggle("dark")
    }
}

const is_mobile = computed(() => {
    return window.innerWidth < 768
})

</script>

<template>
    <header
        class="flex h-(--header-height) shrink-0 items-center gap-2 border-b text-foreground transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-(--header-height)">
        <div class="flex w-full items-center justify-between gap-1 px-4 lg:gap-2 lg:px-6">
            <div class="flex items-center">
                <SidebarTrigger class="-ml-1" v-if="is_mobile" />
                <ArrowLeft @click="router.back()" v-if="router.currentRoute.value.name == 'Content'"
                    class="cursor-pointer" :size=20 />
            </div>
            <Button size="icon" variant="ghost" class="hidden lg:flex" @click="changeAppTheme">
                <Sun />
            </Button>
        </div>
    </header>
</template>
