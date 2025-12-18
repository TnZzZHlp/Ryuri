<script setup lang="ts">
import type { Library, ScanPath } from '@/api';
import { Button } from '@/components/ui/button'
import {
    DialogClose,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { useLibraryStore } from '@/stores/useLibraryStore';
import { onMounted, ref, watch } from 'vue';
import { Trash2, Plus, FolderOpen } from 'lucide-vue-next';
import PathSelector from '@/components/ui/path-selector/PathSelector.vue';
import { toast } from 'vue-sonner';

const props = defineProps<{
    library: Library
}>()

const emit = defineEmits<{
    (e: 'close'): void
}>()

const libraryStore = useLibraryStore()

// Form state
const name = ref(props.library.name)
const scanInterval = ref(props.library.scan_interval)
const watchMode = ref(props.library.watch_mode)
const scanPaths = ref<ScanPath[]>([])
const newPath = ref('')
const loading = ref(false)
const pathsLoading = ref(false)
const showPathSelector = ref(false)

// Reset form when library changes
watch(() => props.library, (newLib) => {
    name.value = newLib.name
    scanInterval.value = newLib.scan_interval
    watchMode.value = newLib.watch_mode
    loadScanPaths()
}, { immediate: false })

// Load scan paths on mount
onMounted(async () => {
    await loadScanPaths()
})

async function loadScanPaths() {
    pathsLoading.value = true
    try {
        scanPaths.value = await libraryStore.fetchScanPaths(props.library.id)
    } catch (e) {
        toast.error('Loading scan paths failed')
    } finally {
        pathsLoading.value = false
    }
}

async function handleSave() {
    if (!name.value.trim()) {
        toast.error('Library name cannot be empty')
        return
    }

    loading.value = true
    try {
        await libraryStore.updateLibrary(props.library.id, {
            name: name.value.trim(),
            scan_interval: scanInterval.value,
            watch_mode: watchMode.value,
        })
        toast.success('Library settings saved')
        emit('close')
    } catch (e) {
        toast.error('Save failed')
    } finally {
        loading.value = false
    }
}

async function handleAddPath() {
    const path = newPath.value.trim()
    if (!path) {
        toast.error('Please enter a scan path')
        return
    }

    try {
        const addedPath = await libraryStore.addScanPath(props.library.id, path)
        scanPaths.value.push(addedPath)
        newPath.value = ''
        toast.success('Scan path added')
    } catch (e) {
        toast.error('Adding scan path failed')
    }
}

async function handleRemovePath(pathId: number) {
    try {
        await libraryStore.removeScanPath(props.library.id, pathId)
        scanPaths.value = scanPaths.value.filter(p => p.id !== pathId)
        toast.success('Scan path removed')
    } catch (e) {
        toast.error('Removing scan path failed')
    }
}

function handlePathSelect(path: string) {
    newPath.value = path
}
</script>

<template>
    <DialogContent class="sm:max-w-[500px]">
        <DialogHeader>
            <DialogTitle>Library Settings</DialogTitle>
            <DialogDescription>
                Edit the library's name, scan settings, and scan paths.
            </DialogDescription>
        </DialogHeader>

        <div class="grid gap-6 py-4">
            <!-- Library Name -->
            <div class="grid gap-2">
                <Label for="library-name">Library Name</Label>
                <Input id="library-name" v-model="name" placeholder="Enter library name" />
            </div>

            <!-- Scan Interval -->
            <div class="grid gap-2">
                <Label for="scan-interval">Auto Scan Interval (minutes)</Label>
                <Input id="scan-interval" v-model.number="scanInterval" type="number" min="0"
                    placeholder="0 to disable auto scan" />
                <p class="text-xs text-muted-foreground">Set to 0 to disable auto scan</p>
            </div>

            <!-- Watch Mode -->
            <div class="flex items-center justify-between">
                <div class="space-y-0.5">
                    <Label for="watch-mode">Watch Mode</Label>
                    <p class="text-xs text-muted-foreground">Enable to monitor file changes in real-time</p>
                </div>
                <Switch id="watch-mode" v-model:checked="watchMode" />
            </div>

            <!-- Scan Paths -->
            <div class="grid gap-2">
                <Label>Scan Paths</Label>
                <div class="flex gap-2">
                    <Input v-model="newPath" placeholder="Enter scan path" class="flex-1"
                        @keyup.enter="handleAddPath" />
                    <Button variant="outline" size="icon" @click="showPathSelector = true" title="Select folder">
                        <FolderOpen class="h-4 w-4" />
                    </Button>
                    <Button variant="outline" size="icon" @click="handleAddPath" :disabled="!newPath.trim()">
                        <Plus class="h-4 w-4" />
                    </Button>
                </div>

                <!-- Path List -->
                <div class="mt-2 space-y-2 max-h-40 overflow-y-auto">
                    <div v-if="pathsLoading" class="text-sm text-muted-foreground text-center py-2">
                        Loading...
                    </div>
                    <div v-else-if="scanPaths.length === 0"
                        class="text-sm text-muted-foreground text-center py-2 border border-dashed rounded-md">
                        <FolderOpen class="h-8 w-8 mx-auto mb-1 opacity-50" />
                        No scan paths available.
                    </div>
                    <div v-else v-for="path in scanPaths" :key="path.id"
                        class="flex items-center justify-between gap-2 p-2 bg-muted/50 rounded-md group">
                        <span class="text-sm truncate flex-1" :title="path.path">{{ path.path }}</span>
                        <Button variant="ghost" size="icon"
                            class="h-6 w-6 opacity-0 group-hover:opacity-100 transition-opacity"
                            @click="handleRemovePath(path.id)">
                            <Trash2 class="h-3 w-3 text-destructive" />
                        </Button>
                    </div>
                </div>
            </div>
        </div>

        <DialogFooter>
            <DialogClose as-child>
                <Button variant="outline">
                    Cancel
                </Button>
            </DialogClose>
            <Button @click="handleSave" :disabled="loading">
                {{ loading ? 'Saving...' : 'Save' }}
            </Button>
        </DialogFooter>
    </DialogContent>

    <PathSelector
        v-model:open="showPathSelector"
        :initial-path="newPath"
        @select="handlePathSelect"
    />
</template>
