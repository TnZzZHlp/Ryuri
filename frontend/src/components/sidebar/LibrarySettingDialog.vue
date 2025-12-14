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
        toast.error('加载扫描路径失败')
    } finally {
        pathsLoading.value = false
    }
}

async function handleSave() {
    if (!name.value.trim()) {
        toast.error('库名称不能为空')
        return
    }

    loading.value = true
    try {
        await libraryStore.updateLibrary(props.library.id, {
            name: name.value.trim(),
            scan_interval: scanInterval.value,
            watch_mode: watchMode.value,
        })
        toast.success('库设置已保存')
        emit('close')
    } catch (e) {
        toast.error('保存失败')
    } finally {
        loading.value = false
    }
}

async function handleAddPath() {
    const path = newPath.value.trim()
    if (!path) {
        toast.error('请输入扫描路径')
        return
    }

    try {
        const addedPath = await libraryStore.addScanPath(props.library.id, path)
        scanPaths.value.push(addedPath)
        newPath.value = ''
        toast.success('扫描路径已添加')
    } catch (e) {
        toast.error('添加扫描路径失败')
    }
}

async function handleRemovePath(pathId: number) {
    try {
        await libraryStore.removeScanPath(props.library.id, pathId)
        scanPaths.value = scanPaths.value.filter(p => p.id !== pathId)
        toast.success('扫描路径已删除')
    } catch (e) {
        toast.error('删除扫描路径失败')
    }
}
</script>

<template>
    <DialogContent class="sm:max-w-[500px]">
        <DialogHeader>
            <DialogTitle>库设置</DialogTitle>
            <DialogDescription>
                编辑库的名称、扫描设置和扫描路径
            </DialogDescription>
        </DialogHeader>

        <div class="grid gap-6 py-4">
            <!-- Library Name -->
            <div class="grid gap-2">
                <Label for="library-name">库名称</Label>
                <Input id="library-name" v-model="name" placeholder="输入库名称" />
            </div>

            <!-- Scan Interval -->
            <div class="grid gap-2">
                <Label for="scan-interval">自动扫描间隔（分钟）</Label>
                <Input id="scan-interval" v-model.number="scanInterval" type="number" min="0"
                    placeholder="0 表示禁用自动扫描" />
                <p class="text-xs text-muted-foreground">设置为 0 禁用自动扫描</p>
            </div>

            <!-- Watch Mode -->
            <div class="flex items-center justify-between">
                <div class="space-y-0.5">
                    <Label for="watch-mode">文件监视模式</Label>
                    <p class="text-xs text-muted-foreground">启用后将实时监视文件变化</p>
                </div>
                <Switch id="watch-mode" v-model:checked="watchMode" />
            </div>

            <!-- Scan Paths -->
            <div class="grid gap-2">
                <Label>扫描路径</Label>
                <div class="flex gap-2">
                    <Input v-model="newPath" placeholder="输入扫描路径" class="flex-1" @keyup.enter="handleAddPath" />
                    <Button variant="outline" size="icon" @click="handleAddPath" :disabled="!newPath.trim()">
                        <Plus class="h-4 w-4" />
                    </Button>
                </div>

                <!-- Path List -->
                <div class="mt-2 space-y-2 max-h-40 overflow-y-auto">
                    <div v-if="pathsLoading" class="text-sm text-muted-foreground text-center py-2">
                        加载中...
                    </div>
                    <div v-else-if="scanPaths.length === 0"
                        class="text-sm text-muted-foreground text-center py-2 border border-dashed rounded-md">
                        <FolderOpen class="h-8 w-8 mx-auto mb-1 opacity-50" />
                        暂无扫描路径
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
                    取消
                </Button>
            </DialogClose>
            <Button @click="handleSave" :disabled="loading">
                {{ loading ? '保存中...' : '保存' }}
            </Button>
        </DialogFooter>
    </DialogContent>
</template>
