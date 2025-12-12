<script setup lang="ts">
import { ChevronRight, Trash2, Settings, LibraryBig, MoreHorizontalIcon, ScanSearch } from "lucide-vue-next"
import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
} from "@/components/ui/collapsible"
import {
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
    SidebarMenuSub,
    SidebarMenuSubItem,
} from "@/components/ui/sidebar"
import { Button } from "@/components/ui/button"
import { Dialog } from '@/components/ui/dialog'
import LibrarySettingDialog from "@/components/sidebar/LibrarySettingDialog.vue"
import { DropdownMenu, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import { useRouter } from "vue-router";
import { computed, onBeforeMount, ref } from "vue"
import { useLibraryStore } from "@/stores/useLibraryStore"
import type { Library } from "@/api"

const libraryStore = useLibraryStore()

const libraries = computed(() => libraryStore.libraries.map((lib) => {
    return {
        url: `/library/${lib.id}`,
        ...lib
    }
}))

const router = useRouter()

const current_lib_id = computed(() => {
    const currentRoute = router.currentRoute.value
    const libraryId = currentRoute.params.libraryId as unknown as number
    return libraryId
})

const settingDialogOpen = ref(false)
const selectedLibrary = ref<Library | null>(null)

const handleOpenSetting = (library: Library) => {
    selectedLibrary.value = library
    settingDialogOpen.value = true
}

const handleScanLibrary = (libraryId: number) => {
    libraryStore.triggerScan(libraryId)
}

onBeforeMount(() => {
    if (!libraryStore.libraries.length) {
        libraryStore.fetchLibraries()
    }
})
</script>

<template>
    <SidebarMenu>
        <Collapsible as-child default-open class="group/collapsible">
            <SidebarMenuItem>
                <CollapsibleTrigger as-child>
                    <SidebarMenuButton tooltip="Libraries">
                        <component :is="LibraryBig" />
                        <span>Libraries</span>
                        <ChevronRight
                            class="ml-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90" />
                    </SidebarMenuButton>
                </CollapsibleTrigger>
                <CollapsibleContent>
                    <SidebarMenuSub>
                        <SidebarMenuSubItem v-for="library in libraries" :key="library.id"
                            class="hover:bg-sidebar-accent rounded-sm duration-300"
                            :class="{ 'bg-sidebar-accent ': current_lib_id == library.id }">
                            <div class="flex items-center justify-between w-full p-1">
                                <router-link :to="library.url" class="w-full  rounded-md">
                                    <span class="pl-2 font-extralight">{{ library.name }}</span>
                                </router-link>
                                <DropdownMenu>
                                    <DropdownMenuTrigger as-child>
                                        <Button variant="ghost" size="icon" class="w-6 h-6" aria-label="More Options">
                                            <MoreHorizontalIcon />
                                        </Button>
                                    </DropdownMenuTrigger>
                                    <DropdownMenuContent align="start">
                                        <DropdownMenuGroup>
                                            <DropdownMenuItem @select="handleScanLibrary(library.id)">
                                                <ScanSearch />
                                                Scan
                                            </DropdownMenuItem>
                                            <DropdownMenuItem @select="handleOpenSetting(library)">
                                                <Settings />
                                                Settings
                                            </DropdownMenuItem>
                                            <DropdownMenuItem variant="destructive">
                                                <Trash2 />
                                                Delete
                                            </DropdownMenuItem>
                                        </DropdownMenuGroup>
                                    </DropdownMenuContent>
                                </DropdownMenu>
                            </div>
                        </SidebarMenuSubItem>
                    </SidebarMenuSub>
                </CollapsibleContent>
            </SidebarMenuItem>
        </Collapsible>
    </SidebarMenu>
    <Dialog v-model:open="settingDialogOpen">
        <LibrarySettingDialog v-if="selectedLibrary" :library="selectedLibrary" @close="settingDialogOpen = false" />
    </Dialog>
</template>