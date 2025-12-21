<script setup lang="ts">
import { ref, watch } from 'vue'
import { Folder, ChevronUp, HardDrive, Loader2 } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { createFilesystemApi, type DirectoryEntry } from '@/api/filesystem'
import { ApiClient } from '@/api/client'
import { useAuthStore } from '@/stores/useAuthStore'
import { toast } from 'vue-sonner'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
    open: boolean
    initialPath?: string
}>()

const emit = defineEmits<{
    (e: 'update:open', value: boolean): void
    (e: 'select', path: string): void
}>()

const { t } = useI18n()
const authStore = useAuthStore()
const apiClient = new ApiClient({
    baseUrl: import.meta.env.VITE_API_BASE_URL || '',
    getToken: () => authStore.token,
})
const fsApi = createFilesystemApi(apiClient)

const currentPath = ref('')
const entries = ref<DirectoryEntry[]>([])
const loading = ref(false)

watch(() => props.open, (isOpen) => {
    if (isOpen) {
        currentPath.value = props.initialPath || ''
        loadDirectory(currentPath.value)
    }
})

async function loadDirectory(path: string) {
    loading.value = true
    try {
        entries.value = await fsApi.listDirectories(path)
        currentPath.value = path
    } catch (e) {
        toast.error(t('path_selector.load_fail'))
        // If loading failed (e.g. permission denied or invalid path), 
        // try to go up or root if not already there
        if (path !== '') {
            // Optional: fallback logic
        }
    } finally {
        loading.value = false
    }
}

function handleNavigate(path: string) {
    loadDirectory(path)
}

function handleUp() {
    if (!currentPath.value) return // Already at root

    // Simple path manipulation for parent
    // Logic needs to handle Windows (C:\) and Unix (/) 

    // Check if it's a Windows drive root like "C:\"
    if (currentPath.value.match(/^[a-zA-Z]:\\$/)) {
        loadDirectory('')
        return
    }

    // Check if it's Unix root
    if (currentPath.value === '/') {
        // Already at root, but our backend treats empty string as "Roots" (drives on windows, root on linux)
        // If we are on linux, / is the top.
        // But let's check what the backend does.
        // Backend: if path is empty -> "/" (Unix) or Drives (Windows).
        // So if we are at "/", going up -> empty string?
        // If we pass empty string on Unix, it returns entries of "/".
        // So on Unix, "/" is the top.
        return
    }

    // General parent logic
    // We can use the 'parent' field from the entries if we had one selected, 
    // but we are navigating UP from the current view.

    // We'll try to strip the last segment
    let separator = currentPath.value.includes('\\') ? '\\' : '/'
    let parent = currentPath.value

    if (parent.endsWith(separator)) {
        parent = parent.slice(0, -1)
    }

    const lastIndex = parent.lastIndexOf(separator)
    if (lastIndex !== -1) {
        // If it's like /home/user -> /home
        // If it's C:\Users -> C:\
        parent = parent.substring(0, lastIndex)
        // Ensure we don't end up with empty string for unix root unless intended
        if (parent === '' && separator === '/') {
            parent = '/'
        }
        // For windows, if we strip C:\Users -> C:, we need C:\
        if (parent.match(/^[a-zA-Z]:$/)) {
            parent += '\\'
        }
    } else {
        // No separator found, go to root
        parent = ''
    }

    loadDirectory(parent)
}

function handleSelect() {
    emit('select', currentPath.value)
    emit('update:open', false)
}

function isDrive(path: string) {
    return path.match(/^[a-zA-Z]:\\$/)
}

function formatPathDisplay(path: string) {
    if (!path) return t('path_selector.computer')
    return path
}
</script>

<template>
    <Dialog :open="open" @update:open="$emit('update:open', $event)">
        <DialogContent class="sm:max-w-[600px] max-h-[80vh] flex flex-col">
            <DialogHeader>
                <DialogTitle>{{ t('path_selector.title') }}</DialogTitle>
                <DialogDescription>
                    {{ t('path_selector.description') }}
                </DialogDescription>
            </DialogHeader>

            <div class="flex items-center gap-2 py-2 px-1">
                <Button variant="outline" size="icon" @click="handleUp" :disabled="!currentPath || currentPath === '/'">
                    <ChevronUp class="h-4 w-4" />
                </Button>
                <div class="flex-1 px-3 py-2 bg-muted rounded-md text-sm font-mono truncate">
                    {{ formatPathDisplay(currentPath) }}
                </div>
            </div>

            <div class="flex-1 border rounded-md h-[300px] overflow-y-auto">
                <div class="p-2 space-y-1">
                    <div v-if="loading" class="flex items-center justify-center py-8">
                        <Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
                    </div>

                    <template v-else>
                        <div v-if="entries.length === 0" class="text-center py-8 text-muted-foreground text-sm">
                            {{ t('path_selector.no_folders') }}
                        </div>

                        <Button v-for="entry in entries" :key="entry.path" variant="ghost"
                            class="w-full justify-start font-normal h-auto py-2" @click="handleNavigate(entry.path)">
                            <component :is="isDrive(entry.path) ? HardDrive : Folder"
                                class="mr-2 h-4 w-4 text-blue-500 fill-blue-500/20" />
                            <span class="truncate">{{ entry.name }}</span>
                        </Button>
                    </template>
                </div>
            </div>

            <DialogFooter>
                <Button variant="outline" @click="$emit('update:open', false)">
                    {{ t('path_selector.cancel') }}
                </Button>
                <Button @click="handleSelect" :disabled="!currentPath">
                    {{ t('path_selector.select_this') }}
                </Button>
            </DialogFooter>
        </DialogContent>
    </Dialog>
</template>
