<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useContentStore } from '@/stores/useContentStore';
import { computed, ref, onBeforeMount } from 'vue';
import { Skeleton } from '@/components/ui/skeleton';
import { Button } from '@/components/ui/button';
import {
    BookOpen,
    User,
    Building2,
    Calendar,
    Hash,
    FileText,
    Star,
    List,
    ChevronRight
} from 'lucide-vue-next';
import type { Chapter } from '@/api/types';
import { toast } from 'vue-sonner';

const router = useRouter();
const libraryId = Number(router.currentRoute.value.params.libraryId);
const contentId = Number(router.currentRoute.value.params.contentId);
const contentStore = useContentStore();
const { getThumbnailUrl, isThumbnailLoading, loadThumbnail } = contentStore;

const content = ref<typeof contentStore.currentContent>(null);
const chapters = ref<Chapter[]>([]);
const contentLoading = ref(true);
const chaptersLoading = ref(false);

onBeforeMount(async () => {
    try {
        // 获取内容详情
        const api = await import('@/api/content');
        const { ApiClient } = await import('@/api/client');
        const { useAuthStore } = await import('@/stores/useAuthStore');
        const authStore = useAuthStore();

        const client = new ApiClient({
            baseUrl: import.meta.env.VITE_API_BASE_URL || '',
            getToken: () => authStore.token,
        });
        const contentApi = api.createContentApi(client);
        const data = await contentApi.get(contentId);
        content.value = data;

        // 加载缩略图
        if (data.has_thumbnail) {
            loadThumbnail(data.id);
        }

        // 加载章节列表
        chaptersLoading.value = true;
        const chapterList = await contentApi.listChapters(contentId);
        chapters.value = chapterList.sort((a, b) => a.sort_order - b.sort_order);
        chaptersLoading.value = false;
    } catch (e) {
        console.error('Failed to fetch content:', e);
    } finally {
        contentLoading.value = false;
    }
});

// 从 metadata 中提取信息
const getMetaValue = (key: string): any => {
    const meta = content.value?.metadata;
    if (meta && typeof meta === 'object' && key in meta) {
        return (meta as Record<string, unknown>)[key] as string;
    }
    return null;
};

const author = computed(() => {
    const infobox = getMetaValue('infobox')
    // 找到 key 为 '作者' 的字段
    return infobox?.find((item: any) => item.key === '作者')?.value || '未知作者';
});
const publisher = computed(() => getMetaValue('publisher'));
const publishDate = computed(() => getMetaValue('publish_date') || getMetaValue('date'));
const isbn = computed(() => getMetaValue('isbn'));
const pageCount = computed(() => getMetaValue('page_count') || getMetaValue('pages'));
const description = computed(() => getMetaValue('description') || getMetaValue('summary'));
const rating = computed(() => {
    const r = getMetaValue('rating');
    return r?.score ?? null;
});
const originalName = computed(() => getMetaValue('name'));

const tags = computed(() => {
    const t = getMetaValue('tags');
    if (Array.isArray(t)) return t as unknown as {
        count: number;
        name: string;
        total_cont: number;
    }[];
});

// 渲染星级评分
const renderStars = (score: number) => {
    const fullStars = Math.floor(score);
    const hasHalf = score - fullStars >= 0.5;
    return { fullStars, hasHalf, emptyStars: 10 - fullStars - (hasHalf ? 1 : 0) };
};

const handleStartReading = (chapterId?: number) => {
    if (chapterId) {
        router.push(`/read/${libraryId}/${contentId}/${chapterId}`);
    } else if (chapters.value.length > 0) {
        // Default to first chapter
        router.push(`/read/${libraryId}/${contentId}/${chapters.value[0]!.id}`);
    } else {
        toast.error('暂无章节');
    }
};
</script>

<template>
    <div class="p-6 mx-auto w-full">
        <!-- Loading State -->
        <div v-if="contentLoading" class="flex flex-col md:flex-row gap-8">
            <div class="w-full md:w-80 shrink-0">
                <Skeleton class="aspect-3/4 w-full rounded-xl" />
                <Skeleton class="h-12 w-full mt-4 rounded-lg" />
            </div>
            <div class="flex-1 space-y-4">
                <Skeleton class="h-10 w-3/4" />
                <Skeleton class="h-6 w-1/3" />
                <Skeleton class="h-6 w-1/4" />
                <Skeleton class="h-32 w-full" />
                <div class="grid grid-cols-2 gap-4">
                    <Skeleton class="h-20" />
                    <Skeleton class="h-20" />
                    <Skeleton class="h-20" />
                    <Skeleton class="h-20" />
                </div>
            </div>
        </div>

        <!-- Content Detail -->
        <div v-else-if="content" class="flex flex-col md:flex-row gap-8">
            <!-- Left: Cover Image -->
            <div class="w-full md:w-80 shrink-0">
                <div class="relative aspect-3/4 w-full overflow-hidden rounded-xl bg-muted shadow-lg">
                    <img v-if="content.has_thumbnail && getThumbnailUrl(content.id)" :src="getThumbnailUrl(content.id)!"
                        :alt="content.title" class="h-full w-full object-cover" />
                    <div v-else-if="content.has_thumbnail && isThumbnailLoading(content.id)"
                        class="flex h-full w-full items-center justify-center">
                        <Skeleton class="h-full w-full" />
                    </div>
                    <div v-else
                        class="flex h-full w-full items-center justify-center bg-linear-to-br from-muted to-muted-foreground/20">
                        <span class="text-6xl text-muted-foreground/50">📚</span>
                    </div>
                </div>

                <!-- Start Reading Button -->
                <Button @click="() => handleStartReading()" class="w-full mt-4 h-12 text-base" size="lg">
                    <BookOpen class="size-5" />
                    开始阅读
                </Button>
            </div>

            <!-- Right: Content Info -->
            <div class="flex-1 min-w-0">
                <!-- Title -->
                <h1 class="text-3xl font-bold text-foreground mb-3">
                    {{ content.title }}
                </h1>
                <h2 v-if="originalName" class="text-lg text-muted-foreground mb-3">
                    {{ originalName }}
                </h2>
                <!-- Author -->
                <div class="flex items-center gap-2 text-muted-foreground mb-3">
                    <User class="size-4" />
                    <span>{{ author }}</span>
                </div>

                <!-- Rating -->
                <div v-if="rating" class="flex items-center gap-2 mb-4">
                    <div class="flex items-center">
                        <template v-for="i in renderStars(rating).fullStars" :key="'full-' + i">
                            <Star class="size-5 text-yellow-500 fill-yellow-500" />
                        </template>
                        <template v-if="renderStars(rating).hasHalf">
                            <Star class="size-5 text-yellow-500" />
                        </template>
                        <template v-for="i in renderStars(rating).emptyStars" :key="'empty-' + i">
                            <Star class="size-5 text-muted-foreground/30" />
                        </template>
                    </div>
                    <span class="text-lg font-medium">{{ rating.toFixed(1) }}</span>
                </div>

                <!-- Description -->
                <p v-if="description" class="text-muted-foreground leading-relaxed mb-6">
                    {{ description }}
                </p>

                <!-- Metadata Grid -->
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 mb-6">
                    <!-- Publisher -->
                    <div v-if="publisher" class="flex flex-col gap-1 p-4 rounded-lg bg-muted/50">
                        <div class="flex items-center gap-2 text-xs text-muted-foreground">
                            <Building2 class="size-4" />
                            <span>出版社</span>
                        </div>
                        <span class="font-medium">{{ publisher }}</span>
                    </div>

                    <!-- Publish Date -->
                    <div v-if="publishDate" class="flex flex-col gap-1 p-4 rounded-lg bg-muted/50">
                        <div class="flex items-center gap-2 text-xs text-muted-foreground">
                            <Calendar class="size-4" />
                            <span>出版时间</span>
                        </div>
                        <span class="font-medium">{{ publishDate }}</span>
                    </div>

                    <!-- ISBN -->
                    <div v-if="isbn" class="flex flex-col gap-1 p-4 rounded-lg bg-muted/50">
                        <div class="flex items-center gap-2 text-xs text-muted-foreground">
                            <Hash class="size-4" />
                            <span>ISBN</span>
                        </div>
                        <span class="font-medium">{{ isbn }}</span>
                    </div>

                    <!-- Page Count -->
                    <div v-if="pageCount" class="flex flex-col gap-1 p-4 rounded-lg bg-muted/50">
                        <div class="flex items-center gap-2 text-xs text-muted-foreground">
                            <FileText class="size-4" />
                            <span>页数</span>
                        </div>
                        <span class="font-medium">{{ pageCount }} 页</span>
                    </div>

                    <!-- Chapter Count (fallback) -->
                    <div v-if="!pageCount" class="flex flex-col gap-1 p-4 rounded-lg bg-muted/50">
                        <div class="flex items-center gap-2 text-xs text-muted-foreground">
                            <FileText class="size-4" />
                            <span>章节数</span>
                        </div>
                        <span class="font-medium">{{ content.chapter_count }} 章</span>
                    </div>
                </div>

                <!-- Tags -->
                <div v-if="tags && tags.length > 0" class="flex flex-wrap gap-2 mb-6">
                    <span v-for="tag in tags" :key="tag.name"
                        class="px-3 py-1 text-sm rounded-full bg-muted text-muted-foreground hover:bg-muted/80 transition-colors">
                        {{ tag.name }}
                    </span>
                </div>

                <!-- Chapters Section -->
                <div class="mt-6">
                    <div class="flex items-center gap-2 mb-4">
                        <List class="size-5" />
                        <h2 class="text-xl font-semibold">章节目录</h2>
                        <span class="text-sm text-muted-foreground">({{ chapters.length }} 章)</span>
                    </div>

                    <!-- Chapters Loading -->
                    <div v-if="chaptersLoading" class="space-y-2">
                        <Skeleton v-for="i in 5" :key="i" class="h-12 w-full rounded-lg" />
                    </div>

                    <!-- Chapters List -->
                    <div v-else-if="chapters.length > 0" class="space-y-2 overflow-y-auto pr-2">
                        <div v-for="chapter in chapters" :key="chapter.id"
                            class="flex items-center justify-between p-3 rounded-lg bg-muted/30 hover:bg-muted/60 transition-colors cursor-pointer group"
                            @click="handleStartReading(chapter.id)">
                            <div class="flex items-center gap-3 min-w-0">
                                <span class="text-sm text-muted-foreground w-8 shrink-0">
                                    {{ chapter.sort_order + 1 }}
                                </span>
                                <span class="truncate">{{ chapter.title }}</span>
                            </div>
                            <ChevronRight
                                class="size-4 text-muted-foreground group-hover:text-foreground transition-colors shrink-0" />
                        </div>
                    </div>

                    <!-- No Chapters -->
                    <div v-else class="text-center py-8 text-muted-foreground">
                        <p>暂无章节</p>
                    </div>
                </div>
            </div>
        </div>

        <!-- Error State -->
        <div v-else class="flex flex-col items-center justify-center py-20 text-muted-foreground">
            <p class="text-lg">内容不存在或加载失败</p>
            <Button variant="outline" class="mt-4" @click="router.back()">
                返回上一页
            </Button>
        </div>
    </div>
</template>
