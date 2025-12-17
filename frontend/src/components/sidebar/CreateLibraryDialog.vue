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
        toast.error('Library name cannot be empty')
        return
    }

    loading.value = true
    try {
        await libraryStore.createLibrary({
            name: name.value.trim(),
            scan_interval: scanInterval.value,
            watch_mode: watchMode.value,
        })
        toast.success('Library created successfully')
        emit('close')
    } catch (e) {
        toast.error('Creation failed')
    } finally {
        loading.value = false
    }
}
</script>

<template>
    <DialogContent class="sm:max-w-[500px]">
        <DialogHeader>
            <DialogTitle>Create New Library</DialogTitle>
            <DialogDescription>
                Enter the name and settings for the new library.
            </DialogDescription>
        </DialogHeader>

        <div class="grid gap-6 py-4">
            <!-- Library Name -->
            <div class="grid gap-2">
                <Label for="library-name">Library Name</Label>
                <Input id="library-name" v-model="name" placeholder="Enter library name" @keyup.enter="handleCreate" />
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
        </div>

        <DialogFooter>
            <DialogClose as-child>
                <Button variant="outline">
                    Cancel
                </Button>
            </DialogClose>
            <Button @click="handleCreate" :disabled="loading">
                {{ loading ? 'Creating...' : 'Create' }}
            </Button>
        </DialogFooter>
    </DialogContent>
</template>
