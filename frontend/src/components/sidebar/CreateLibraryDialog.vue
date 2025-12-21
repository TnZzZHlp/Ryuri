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
import { FolderOpen } from 'lucide-vue-next'
import PathSelector from '@/components/ui/path-selector/PathSelector.vue'
import { useI18n } from 'vue-i18n'

const emit = defineEmits<{
    (e: 'close'): void
}>()

const libraryStore = useLibraryStore()
const { t } = useI18n()

// Form state
const name = ref('')
const scanInterval = ref(0)
const watchMode = ref(false)
const path = ref('')
const showPathSelector = ref(false)
const loading = ref(false)

async function handleCreate() {
    if (!name.value.trim()) {
        toast.error(t('library.name_required'))
        return
    }

    loading.value = true
    try {
        const newLib = await libraryStore.createLibrary({
            name: name.value.trim(),
            scan_interval: scanInterval.value,
            watch_mode: watchMode.value,
        })

        if (path.value.trim()) {
            try {
                await libraryStore.addScanPath(newLib.id, path.value.trim())
            } catch (e) {
                toast.error(t('library.scan_path_fail'))
            }
        }

        toast.success(t('library.create_success'))
        emit('close')
    } catch (e) {
        toast.error(t('library.create_fail'))
    } finally {
        loading.value = false
    }
}

function handlePathSelect(selectedPath: string) {
    path.value = selectedPath
}
</script>

<template>
    <DialogContent class="sm:max-w-[500px]">
        <DialogHeader>
            <DialogTitle>{{ t('library.create_title') }}</DialogTitle>
            <DialogDescription>
                {{ t('library.create_desc') }}
            </DialogDescription>
        </DialogHeader>

        <div class="grid gap-6 py-4">
            <!-- Library Name -->
            <div class="grid gap-2">
                <Label for="library-name">{{ t('library.name_label') }}</Label>
                <Input id="library-name" v-model="name" :placeholder="t('library.name_placeholder')" @keyup.enter="handleCreate" />
            </div>

            <!-- Initial Scan Path -->
            <div class="grid gap-2">
                <Label for="library-path">{{ t('library.path_label') }}</Label>
                <div class="flex gap-2">
                    <Input id="library-path" v-model="path" :placeholder="t('library.path_placeholder')" class="flex-1" />
                    <Button variant="outline" size="icon" @click="showPathSelector = true" :title="t('library.path_select_tooltip')">
                        <FolderOpen class="h-4 w-4" />
                    </Button>
                </div>
            </div>

            <!-- Scan Interval -->
            <div class="grid gap-2">
                <Label for="scan-interval">{{ t('library.scan_interval_label') }}</Label>
                <Input id="scan-interval" v-model.number="scanInterval" type="number" min="0"
                    :placeholder="t('library.scan_interval_placeholder')" />
                <p class="text-xs text-muted-foreground">{{ t('library.scan_interval_help') }}</p>
            </div>

            <!-- Watch Mode -->
            <div class="flex items-center justify-between">
                <div class="space-y-0.5">
                    <Label for="watch-mode">{{ t('library.watch_mode_label') }}</Label>
                    <p class="text-xs text-muted-foreground">{{ t('library.watch_mode_help') }}</p>
                </div>
                <Switch id="watch-mode" v-model:checked="watchMode" />
            </div>
        </div>

        <DialogFooter>
            <DialogClose as-child>
                <Button variant="outline">
                    {{ t('common.cancel') }}
                </Button>
            </DialogClose>
            <Button @click="handleCreate" :disabled="loading">
                {{ loading ? t('library.creating_btn') : t('library.create_btn') }}
            </Button>
        </DialogFooter>
    </DialogContent>

    <PathSelector
        v-model:open="showPathSelector"
        :initial-path="path"
        @select="handlePathSelect"
    />
</template>
