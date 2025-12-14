<script setup lang="ts">
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
import { useLibraryStore } from '@/stores/useLibraryStore'
import { ref } from 'vue'
import { toast } from 'vue-sonner'

const emit = defineEmits<{
    (e: 'close'): void
}>()

const libraryStore = useLibraryStore()

// Form state
const name = ref('')
const scanInterval = ref(0)
const watchMode = ref(false)
const loading = ref(false)

async function handleCreate() {
    if (!name.value.trim()) {
        toast.error('库名称不能为空')
        return
    }

    loading.value = true
    try {
        await libraryStore.createLibrary({
            name: name.value.trim(),
            scan_interval: scanInterval.value,
            watch_mode: watchMode.value,
        })
        toast.success('库创建成功')
        emit('close')
    } catch (e) {
        toast.error('创建失败')
    } finally {
        loading.value = false
    }
}
</script>

<template>
    <DialogContent class="sm:max-w-[500px]">
        <DialogHeader>
            <DialogTitle>创建新库</DialogTitle>
            <DialogDescription>
                输入新库的名称和设置
            </DialogDescription>
        </DialogHeader>

        <div class="grid gap-6 py-4">
            <!-- Library Name -->
            <div class="grid gap-2">
                <Label for="library-name">库名称</Label>
                <Input id="library-name" v-model="name" placeholder="输入库名称" @keyup.enter="handleCreate" />
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
        </div>

        <DialogFooter>
            <DialogClose as-child>
                <Button variant="outline">
                    取消
                </Button>
            </DialogClose>
            <Button @click="handleCreate" :disabled="loading">
                {{ loading ? '创建中...' : '创建' }}
            </Button>
        </DialogFooter>
    </DialogContent>
</template>
