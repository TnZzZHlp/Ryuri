<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useContentStore } from '@/stores/useContentStore';
import { useAuthStore } from '@/stores/useAuthStore';
import { onBeforeMount, computed, ref, watch } from 'vue';
import { Skeleton } from '@/components/ui/skeleton';

const router = useRouter();
const library_id: number = router.currentRoute.value.params.id as unknown as number;
const { contents, fetchContents, loading } = useContentStore();
const authStore = useAuthStore();

// 缩略图缓存
const thumbnailUrls = ref<Map<number, string>>(new Map());

onBeforeMount(() => {
    if (!contents.get(library_id)) {
        fetchContents(library_id);
    }
})

const books = computed(() => contents.get(library_id) || []);

// 加载缩略图（带Authorization）
async function loadThumbnail(contentId: number): Promise<string | null> {
    if (thumbnailUrls.value.has(contentId)) {
        return thumbnailUrls.value.get(contentId)!;
    }

    try {
        const response = await fetch(`/api/contents/${contentId}/thumbnail`, {
            headers: {
                'Authorization': `Bearer ${authStore.token}`
            }
        });

        if (!response.ok) return null;

        const blob = await response.blob();
        const url = URL.createObjectURL(blob);
        thumbnailUrls.value.set(contentId, url);
        return url;
    } catch {
        return null;
    }
}

// 监听books变化，预加载缩略图
watch(books, async (newBooks) => {
    for (const book of newBooks) {
        if (book.has_thumbnail && !thumbnailUrls.value.has(book.id)) {
            loadThumbnail(book.id);
        }
    }
}, { immediate: true });

// 从metadata中获取作者
const getAuthor = (metadata: unknown): string => {
    if (metadata && typeof metadata === 'object' && 'author' in metadata) {
        return (metadata as { author: string }).author;
    }
    return '';
};
</script>

<template>
    <div class="p-6">
        <!-- 书籍网格 -->
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-6">
            <!-- 加载骨架屏 -->
            <template v-if="loading">
                <div v-for="i in 12" :key="i" class="flex flex-col gap-3">
                    <Skeleton class="aspect-3/4 w-full rounded-lg" />
                    <Skeleton class="h-4 w-3/4" />
                    <Skeleton class="h-3 w-1/2" />
                </div>
            </template>

            <!-- 书籍卡片 -->
            <template v-else>
                <div v-for="book in books" :key="book.id" class="group cursor-pointer"
                    @click="router.push(`/content/${book.id}`)">
                    <!-- 封面图片容器 -->
                    <div class="relative aspect-3/4 w-full overflow-hidden rounded-lg bg-muted">
                        <!-- 封面图片 -->
                        <img v-if="book.has_thumbnail && thumbnailUrls.get(book.id)" :src="thumbnailUrls.get(book.id)"
                            :alt="book.title"
                            class="h-full w-full object-cover transition-transform duration-300 group-hover:scale-105" />
                        <!-- 加载中或无封面占位 -->
                        <div v-else
                            class="flex h-full w-full items-center justify-center bg-linear-to-br from-muted to-muted-foreground/20">
                            <span class="text-4xl text-muted-foreground/50">📚</span>
                        </div>

                        <!-- 底部绿色装饰条 -->
                        <div class="absolute bottom-0 left-0 right-0 h-1.5 bg-emerald-400" />
                    </div>

                    <!-- 书籍信息 -->
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
                </div>
            </template>

            <!-- 空状态 -->
            <div v-if="!loading && books.length === 0"
                class="col-span-full flex flex-col items-center justify-center py-20 text-muted-foreground">
                <span class="text-6xl mb-4">📖</span>
                <p class="text-lg">暂无书籍</p>
                <p class="text-sm">扫描书库以添加内容</p>
            </div>
        </div>
    </div>
</template>
