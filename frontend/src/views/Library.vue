<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useContentStore } from '@/stores/useContentStore';
import { onActivated, computed, onBeforeMount } from 'vue';
import { Skeleton } from '@/components/ui/skeleton';

const router = useRouter();
const library_id: number = router.currentRoute.value.params.id as unknown as number;
const contentStore = useContentStore();
const { contents, fetchContents, loading, getThumbnailUrl, isThumbnailLoading } = contentStore;

onBeforeMount(() => {
    if (!contents.get(library_id)) {
        fetchContents(library_id);
    }
})

const books = computed(() => contents.get(library_id) || []);

// Retrieve the author from the metadata.
const getAuthor = (metadata: unknown): string => {
    if (metadata && typeof metadata === 'object' && 'author' in metadata) {
        return (metadata as { author: string }).author;
    }
    return '';
};
</script>

<template>
    <div class="p-6">
        <!-- Book Grid -->
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-6">
            <!-- Loading skeleton screen -->
            <template v-if="loading">
                <div v-for="i in 12" :key="i" class="flex flex-col gap-3">
                    <Skeleton class="aspect-3/4 w-full rounded-lg" />
                    <Skeleton class="h-4 w-3/4" />
                    <Skeleton class="h-3 w-1/2" />
                </div>
            </template>

            <!-- Book Card -->
            <template v-else>
                <router-link v-for="book in books" :key="book.id" :to="`/content/${book.id}`" class="group block">
                    <!-- Cover Image Container -->
                    <div
                        class="relative aspect-3/4 w-full overflow-hidden rounded-lg bg-muted hover:shadow-sm duration-300">
                        <!-- Cover image -->
                        <img v-if="book.has_thumbnail && getThumbnailUrl(book.id)" :src="getThumbnailUrl(book.id)!"
                            :alt="book.title" class="h-full w-full object-cover transition-transform" />
                        <!-- Loading placeholder -->
                        <div v-else-if="book.has_thumbnail && isThumbnailLoading(book.id)"
                            class="flex h-full w-full items-center justify-center bg-linear-to-br from-muted to-muted-foreground/20">
                            <Skeleton class="h-full w-full" />
                        </div>
                        <!-- No cover placeholder -->
                        <div v-else
                            class="flex h-full w-full items-center justify-center bg-linear-to-br from-muted to-muted-foreground/20">
                            <span class="text-4xl text-muted-foreground/50">📚</span>
                        </div>
                    </div>

                    <!-- Book Information -->
                    <div class="mt-3 space-y-1">
                        <h3
                            class="line-clamp-2 text-sm font-medium leading-tight text-foreground group-hover:text-primary transition-colors">
                            {{ book.title }}
                        </h3>
                        <p v-if="getAuthor(book.metadata)" class="text-xs text-muted-foreground truncate">
                            {{ getAuthor(book.metadata) }}
                        </p>
                        <p v-else class="text-xs text-muted-foreground">
                            {{ book.chapter_count }} 章节
                        </p>
                    </div>
                </router-link>
            </template>

            <!-- 空状态 -->
            <div v-if="!loading && books.length === 0"
                class="col-span-full flex flex-col items-center justify-center py-20 text-muted-foreground">
                <p class="text-lg">暂无书籍</p>
                <p class="text-sm">扫描书库以添加内容</p>
            </div>
        </div>
    </div>
</template>
