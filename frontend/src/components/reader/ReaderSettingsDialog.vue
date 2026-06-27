<script setup lang="ts">
import { Button } from '@/components/ui/button'
import {
    DialogClose,
    DialogContent,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { AlignJustify, Columns } from 'lucide-vue-next'
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { ReaderMode } from '@/stores/useReaderStore'

const props = defineProps<{
    readerMode: ReaderMode
    preloadBuffer: number
    minPreloadBuffer: number
    maxPreloadBuffer: number
}>()

const emit = defineEmits<{
    (e: 'close'): void
    (e: 'update:readerMode', mode: ReaderMode): void
    (e: 'update:preloadBuffer', value: number): void
}>()

const { t } = useI18n()
const preloadBufferValue = ref(props.preloadBuffer)

const handleModeChange = (mode: ReaderMode) => {
    emit('update:readerMode', mode)
}

const normalizePreloadBuffer = () => {
    if (!Number.isFinite(preloadBufferValue.value)) {
        preloadBufferValue.value = props.preloadBuffer
        return
    }

    preloadBufferValue.value = Math.min(
        props.maxPreloadBuffer,
        Math.max(props.minPreloadBuffer, Math.floor(preloadBufferValue.value)),
    )
}

watch(
    () => props.preloadBuffer,
    (value) => {
        preloadBufferValue.value = value
    },
)

watch(preloadBufferValue, (value) => {
    if (!Number.isFinite(value)) return
    emit('update:preloadBuffer', value)
})
</script>

<template>
    <DialogContent class="sm:max-w-100">
        <DialogHeader>
            <DialogTitle>{{ t('reader.settings_title') }}</DialogTitle>
        </DialogHeader>

        <div class="grid gap-6 py-4">
            <!-- Reading Mode Section -->
            <div class="space-y-3">
                <Label class="text-base font-medium">{{ t('reader.reading_mode') }}</Label>
                <div class="grid grid-cols-2 gap-4">
                    <Button variant="outline" :class="[
                        'flex flex-col items-center justify-center h-24 gap-2',
                        readerMode === 'paged' ? 'border-primary bg-primary/5' : ''
                    ]" @click="handleModeChange('paged')">
                        <Columns class="h-6 w-6" />
                        <span class="text-sm font-medium">{{ t('reader.mode_paged') }}</span>
                    </Button>
                    <Button variant="outline" :class="[
                        'flex flex-col items-center justify-center h-24 gap-2',
                        readerMode === 'scroll' ? 'border-primary bg-primary/5' : ''
                    ]" @click="handleModeChange('scroll')">
                        <AlignJustify class="h-6 w-6" />
                        <span class="text-sm font-medium">{{ t('reader.mode_scroll') }}</span>
                    </Button>
                </div>
            </div>

            <!-- Preload Pages -->
            <div class="grid gap-2">
                <Label for="preload-pages" class="text-base font-medium">
                    {{ t('reader.preload_pages') }}
                </Label>
                <Input
                    id="preload-pages"
                    v-model.number="preloadBufferValue"
                    type="number"
                    :min="minPreloadBuffer"
                    :max="maxPreloadBuffer"
                    @blur="normalizePreloadBuffer"
                />
                <p class="text-xs text-muted-foreground">
                    {{ t('reader.preload_pages_help', { min: minPreloadBuffer, max: maxPreloadBuffer }) }}
                </p>
            </div>
        </div>

        <DialogFooter>
            <DialogClose as-child>
                <Button variant="outline" @click="emit('close')">
                    {{ t('reader.close_btn') }}
                </Button>
            </DialogClose>
        </DialogFooter>
    </DialogContent>
</template>
