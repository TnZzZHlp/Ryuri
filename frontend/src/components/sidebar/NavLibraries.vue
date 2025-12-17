<script setup lang="ts">
import { ChevronRight, Trash2, Settings, LibraryBig, MoreHorizontalIcon, ScanSearch, Plus } from "lucide-vue-next"
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
import CreateLibraryDialog from "@/components/sidebar/CreateLibraryDialog.vue"
import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { DropdownMenu, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import { useRouter } from "vue-router";
import { computed, onBeforeMount, ref } from "vue"
import { useLibraryStore } from "@/stores/useLibraryStore"
import { useScanTaskStore } from "@/stores/useScanTaskStore"
import type { Library } from "@/api"
import { toast } from "vue-sonner"

const libraryStore = useLibraryStore()
const scanTaskStore = useScanTaskStore()

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
const createDialogOpen = ref(false)
const deleteDialogOpen = ref(false)
const selectedLibrary = ref<Library | null>(null)
const libraryToDelete = ref<Library | null>(null)

const handleOpenSetting = (library: Library) => {
    selectedLibrary.value = library
    settingDialogOpen.value = true
}

const handleOpenDelete = (library: Library) => {
    libraryToDelete.value = library
    deleteDialogOpen.value = true
}

const handleConfirmDelete = async () => {
    if (!libraryToDelete.value) return

    try {
        await libraryStore.deleteLibrary(libraryToDelete.value.id)
        toast.success('Library deleted successfully')
    } catch (e) {
        toast.error('Failed to delete library', {
            description: e instanceof Error ? e.message : 'Unknown error'
        })
    } finally {
        deleteDialogOpen.value = false
        libraryToDelete.value = null
    }
}

const handleScanLibrary = async (libraryId: number) => {
    try {
        await scanTaskStore.triggerScan(libraryId)
        toast.success('Scan task triggered successfully')
    } catch (e) {
        toast.error('Failed to trigger scan task', {
            description: e instanceof Error ? e.message : 'Unknown error'
        })
    }
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
                        <div class="ml-auto flex items-center gap-1">
                            <div role="button" tabindex="0"
                                class="hover:bg-sidebar-accent hover:text-sidebar-accent-foreground rounded-sm p-0.5 transition-colors"
                                @click.stop="createDialogOpen = true">
                                <Plus class="h-4 w-4" />
                            </div>
                            <ChevronRight
                                class="transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90" />
                        </div>
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
                                            <DropdownMenuItem variant="destructive" @select="handleOpenDelete(library)">
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
    <Dialog v-model:open="createDialogOpen">
        <CreateLibraryDialog @close="createDialogOpen = false" />
    </Dialog>
    <AlertDialog v-model:open="deleteDialogOpen">
        <AlertDialogContent>
            <AlertDialogHeader>
                <AlertDialogTitle>Confirm Delete Library?</AlertDialogTitle>
                <AlertDialogDescription>
                    This action will permanently delete the library "{{ libraryToDelete?.name }}" and all its associated
                    data (including scan history, progress, etc.).
                    Files in the file system will not be deleted.
                </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
                <AlertDialogCancel>Cancel</AlertDialogCancel>
                <AlertDialogAction @click="handleConfirmDelete"
                    class="bg-destructive text-destructive-foreground hover:bg-destructive/90">
                    Delete
                </AlertDialogAction>
            </AlertDialogFooter>
        </AlertDialogContent>
    </AlertDialog>
</template>