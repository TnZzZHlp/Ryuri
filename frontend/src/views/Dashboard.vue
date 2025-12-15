<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useAuthStore } from '@/stores/useAuthStore';
import { useContentStore } from '@/stores/useContentStore';
import { createProgressApi } from '@/api/progress';
import { ApiClient } from '@/api/client';
import type { ContentResponse } from '@/api/types';
import { Skeleton } from '@/components/ui/skeleton';

const authStore = useAuthStore();
const contentStore = useContentStore();
const { getThumbnailUrl, isThumbnailLoading, loadThumbnail } = contentStore;

const recentBooks = ref<ContentResponse[]>([]);
const loading = ref(true);

const getAuthor = (metadata: unknown): string => {
    if (metadata && typeof metadata === 'object' && 'author' in metadata) {
        return (metadata as { author: string }).author;
    }
    return '';
};

onMounted(async () => {
    const client = new ApiClient({
        baseUrl: import.meta.env.VITE_API_BASE_URL || "",
        getToken: () => authStore.token,
    });
    const progressApi = createProgressApi(client);

    try {
        recentBooks.value = await progressApi.getRecentProgress(5);
        // Preload thumbnails
        recentBooks.value.forEach(book => {
            if (book.has_thumbnail) {
                loadThumbnail(book.id);
            }
        });
    } catch (e) {
        console.error("Failed to load recent books", e);
    } finally {
        loading.value = false;
    }
});
</script>

<template>
    <div class="p-6">
        <h2 class="text-2xl font-bold tracking-tight mb-6">Recently Read</h2>
        
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-6">
            <!-- Loading -->
            <template v-if="loading">
                <div v-for="i in 5" :key="i" class="flex flex-col gap-3">
                    <Skeleton class="aspect-3/4 w-full rounded-lg" />
                    <Skeleton class="h-4 w-3/4" />
                    <Skeleton class="h-3 w-1/2" />
                </div>
            </template>

            <!-- Book Cards -->
            <template v-else>
                <router-link v-for="book in recentBooks" :key="book.id" 
                    :to="`/library/${book.library_id}/content/${book.id}`"
                    class="group block">
                    <!-- Cover Image Container -->
                    <div class="relative aspect-3/4 w-full overflow-hidden rounded-lg bg-muted hover:shadow-sm duration-300">
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
                        <h3 class="line-clamp-2 text-sm font-medium leading-tight text-foreground group-hover:text-primary transition-colors">
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

             <!-- Empty State -->
            <div v-if="!loading && recentBooks.length === 0"
                class="col-span-full flex flex-col items-center justify-center py-10 text-muted-foreground">
                <p>No recent reading history</p>
            </div>
        </div>
    </div>
</template>
