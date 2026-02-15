<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { ChevronDown, ChevronRight, X } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'
import { isLooseEpubPathMatch, normalizeEpubPath, splitEpubSrc } from '@/lib/utils'
import type { EpubTocItem } from '@/api/types'

interface Props {
    show: boolean
    items: EpubTocItem[]
    loading: boolean
    currentSpinePath: string
}

interface FlatTocItem {
    key: string
    depth: number
    item: EpubTocItem
    hasChildren: boolean
    isExpanded: boolean
    isActive: boolean
}

const props = defineProps<Props>()
const emit = defineEmits<{
    close: []
    select: [item: EpubTocItem]
}>()

const { t } = useI18n()
const expandedKeys = ref<Set<string>>(new Set())

const getItemPath = (item: EpubTocItem): string => {
    const parsed = splitEpubSrc(item.src || '')
    return normalizeEpubPath(parsed.path).split('?')[0] ?? ''
}

const makeItemKey = (item: EpubTocItem, parentKey: string, index: number): string => {
    return `${parentKey}/${index}:${getItemPath(item) || item.src || item.title}`
}

const markActiveAncestorsExpanded = (
    items: EpubTocItem[],
    parentKey: string,
    currentPath: string,
) => {
    items.forEach((item, index) => {
        const key = makeItemKey(item, parentKey, index)
        const itemPath = getItemPath(item)
        const isActiveNode = itemPath && isLooseEpubPathMatch(itemPath, currentPath)
        if (isActiveNode) {
            const segments = key.split('/')
            const running: string[] = []
            segments.forEach((segment) => {
                if (!segment) return
                running.push(segment)
                expandedKeys.value.add(running.join('/'))
            })
        }
        if (item.children.length > 0) {
            markActiveAncestorsExpanded(item.children, key, currentPath)
        }
    })
}

watch(
    () => [props.items, props.currentSpinePath],
    () => {
        const normalizedCurrent = normalizeEpubPath(props.currentSpinePath)
        const nextExpanded = new Set<string>()
        props.items.forEach((item, index) => {
            nextExpanded.add(makeItemKey(item, '', index))
        })
        expandedKeys.value = nextExpanded
        if (normalizedCurrent) {
            markActiveAncestorsExpanded(props.items, '', normalizedCurrent)
        }
    },
    { immediate: true, deep: true },
)

const visibleItems = computed<FlatTocItem[]>(() => {
    const normalizedCurrent = normalizeEpubPath(props.currentSpinePath)
    const list: FlatTocItem[] = []

    const walk = (items: EpubTocItem[], depth: number, parentKey: string) => {
        items.forEach((item, index) => {
            const key = makeItemKey(item, parentKey, index)
            const hasChildren = item.children.length > 0
            const isExpanded = expandedKeys.value.has(key)
            const itemPath = getItemPath(item)
            const isActive = !!itemPath && !!normalizedCurrent && isLooseEpubPathMatch(itemPath, normalizedCurrent)
            list.push({
                key,
                depth,
                item,
                hasChildren,
                isExpanded,
                isActive,
            })
            if (hasChildren && isExpanded) {
                walk(item.children, depth + 1, key)
            }
        })
    }

    walk(props.items, 0, '')
    return list
})

const toggleExpand = (key: string) => {
    const next = new Set(expandedKeys.value)
    if (next.has(key)) {
        next.delete(key)
    } else {
        next.add(key)
    }
    expandedKeys.value = next
}

const handleSelect = (entry: FlatTocItem) => {
    if (entry.hasChildren && !getItemPath(entry.item)) {
        toggleExpand(entry.key)
        return
    }
    emit('select', entry.item)
}
</script>

<template>
    <div
        v-if="show"
        class="fixed top-16 right-0 bottom-16 w-[22rem] max-w-[90vw] bg-black/95 border-l border-white/10 shadow-2xl z-[60] flex flex-col"
    >
        <div class="flex items-center justify-between p-4 border-b border-white/10">
            <h3 class="text-sm font-semibold text-white">{{ t('reader.toc_title') }}</h3>
            <button
                type="button"
                class="h-8 w-8 inline-flex items-center justify-center text-gray-300 hover:text-white hover:bg-white/10 rounded"
                @click="emit('close')"
            >
                <X class="h-4 w-4" />
            </button>
        </div>

        <div class="flex-1 overflow-auto py-2">
            <div v-if="loading" class="px-4 py-3 text-sm text-gray-400">{{ t('reader.toc_loading') }}</div>
            <div v-else-if="visibleItems.length === 0" class="px-4 py-3 text-sm text-gray-400">
                {{ t('reader.toc_empty') }}
            </div>
            <template v-else>
                <button
                    v-for="entry in visibleItems"
                    :key="entry.key"
                    type="button"
                    class="w-full text-left flex items-center gap-2 px-2 py-2 text-sm hover:bg-white/8 transition-colors"
                    :class="entry.isActive ? 'bg-white/10 text-white' : 'text-gray-200'"
                    :style="{ paddingLeft: `${entry.depth * 16 + 8}px` }"
                    @click="handleSelect(entry)"
                >
                    <span
                        class="h-5 w-5 inline-flex items-center justify-center rounded hover:bg-white/10"
                        @click.stop="entry.hasChildren ? toggleExpand(entry.key) : undefined"
                    >
                        <ChevronDown v-if="entry.hasChildren && entry.isExpanded" class="h-4 w-4" />
                        <ChevronRight v-else-if="entry.hasChildren" class="h-4 w-4" />
                    </span>
                    <span class="truncate">{{ entry.item.title }}</span>
                </button>
            </template>
        </div>
    </div>
</template>
