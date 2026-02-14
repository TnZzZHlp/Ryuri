<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { ReaderContainer } from '@/components/reader'

const route = useRoute()

const libraryId = computed(() => Number(route.params.libraryId))
const contentId = computed(() => Number(route.params.contentId))
const chapterId = computed(() => Number(route.params.chapterId))
</script>

<template>
    <ReaderContainer
        :library-id="libraryId"
        :content-id="contentId"
        :chapter-id="chapterId"
    />
</template>
