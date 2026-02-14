<script setup lang="ts">
import { Button } from '@/components/ui/button'
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
    ArrowLeft,
    ChevronLeft,
    ChevronRight,
    Settings,
    AlignJustify,
    Columns,
} from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import ReaderProgress from './ReaderProgress.vue'
import type { Chapter } from '@/api/types'

const { t } = useI18n()

interface Props {
    show: boolean
    currentChapter: Chapter | undefined
    currentChapterIndex: number
    chapters: Chapter[]
    isNovel: boolean
    readerMode: 'scroll' | 'paged'
    readingProgress: number
    currentPage: number
    epubCurrentSpineIndex: number
    epubSpineLength: number
    prevChapter: Chapter | null | undefined
    nextChapter: Chapter | null | undefined
}

const props = defineProps<Props>()

const emit = defineEmits<{
    navigateBack: []
    navigateToChapter: [chapter: Chapter]
    setMode: [mode: 'scroll' | 'paged']
}>()
</script>

<template>
    <!-- Top Bar -->
    <div
        class="fixed top-0 left-0 right-0 h-16 bg-black/80 backdrop-blur-sm border-b border-white/10 flex items-center px-4 transition-transform duration-300 z-50"
        :class="show ? 'translate-y-0' : '-translate-y-full'"
    >
        <Button variant="ghost" size="icon" @click="emit('navigateBack')">
            <ArrowLeft class="h-5 w-5 text-white" />
        </Button>
        <div class="ml-4 flex-1 overflow-hidden">
            <h1 class="text-sm font-medium truncate text-white">
                {{ currentChapter?.title || t('reader.chapter_title_fallback', { index: currentChapterIndex + 1 }) }}
            </h1>
        </div>

        <!-- Settings Menu (hidden for novels) -->
        <DropdownMenu v-if="!isNovel">
            <DropdownMenuTrigger as-child>
                <Button variant="ghost" size="icon">
                    <Settings class="h-5 w-5 text-white" />
                </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
                <DropdownMenuLabel>{{ t('reader.reading_mode') }}</DropdownMenuLabel>
                <DropdownMenuSeparator />
                <DropdownMenuItem @click="emit('setMode', 'scroll')">
                    <AlignJustify class="mr-2 h-4 w-4" />
                    <span>{{ t('reader.mode_scroll') }}</span>
                    <span v-if="readerMode === 'scroll'" class="ml-auto text-xs">✓</span>
                </DropdownMenuItem>
                <DropdownMenuItem @click="emit('setMode', 'paged')">
                    <Columns class="mr-2 h-4 w-4" />
                    <span>{{ t('reader.mode_paged') }}</span>
                    <span v-if="readerMode === 'paged'" class="ml-auto text-xs">✓</span>
                </DropdownMenuItem>
            </DropdownMenuContent>
        </DropdownMenu>
    </div>

    <!-- Bottom Bar (Navigation) -->
    <div
        class="fixed bottom-0 left-0 right-0 h-16 bg-black/80 backdrop-blur-sm border-t border-white/10 flex items-center justify-between px-4 transition-transform duration-300 z-50"
        :class="show ? 'translate-y-0' : 'translate-y-full'"
    >
        <ReaderProgress :value="readingProgress" />

        <Button
            variant="ghost"
            size="sm"
            :disabled="!prevChapter"
            @click="prevChapter && emit('navigateToChapter', prevChapter)"
            class="text-white hover:text-white/80"
        >
            <ChevronLeft class="mr-1 h-4 w-4" /> {{ t('reader.prev') }}
        </Button>

        <div class="flex flex-col items-center">
            <span class="text-xs text-gray-400">
                <template v-if="isNovel">
                    {{ `${epubCurrentSpineIndex + 1} / ${epubSpineLength}` }}
                </template>
                <template v-else>
                    {{ readerMode === 'paged' 
                        ? (currentPage + 1) + ' / ' + (props.currentChapter?.page_count || '?')
                        : `${currentChapterIndex + 1} / ${chapters.length}` 
                    }}
                </template>
            </span>
        </div>

        <Button
            variant="ghost"
            size="sm"
            :disabled="!nextChapter"
            @click="nextChapter && emit('navigateToChapter', nextChapter)"
            class="text-white hover:text-white/80"
        >
            {{ t('reader.next') }}
            <ChevronRight class="ml-1 h-4 w-4" />
        </Button>
    </div>
</template>
