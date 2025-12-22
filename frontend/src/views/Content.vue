<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useContentStore } from '@/stores/useContentStore';
import { computed, ref, onBeforeMount } from 'vue';
import { Skeleton } from '@/components/ui/skeleton';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import {
    BookOpen,
    User,
    Building2,
    Calendar,
    Hash,
    FileText,
    Star,
    List,
    ChevronRight,
    Clapperboard,
    Music,
    Palette,
    Tv,
    Layers
} from 'lucide-vue-next';
import type { Chapter } from '@/api/types';
import { toast } from 'vue-sonner';
import { useI18n } from 'vue-i18n';

const router = useRouter();
const libraryId = Number(router.currentRoute.value.params.libraryId);
const contentId = Number(router.currentRoute.value.params.contentId);
const contentStore = useContentStore();
const { getThumbnailUrl, isThumbnailLoading, loadThumbnail } = contentStore;
const { t } = useI18n();

const content = computed(() => {
    if (contentStore.currentContent?.id === contentId) {
        return contentStore.currentContent;
    }
    return null;
});
const chapters = ref<Chapter[]>([]);
const contentLoading = ref(true);
const chaptersLoading = ref(false);
const lastReadChapterId = ref<number | null>(null);
const progressStats = ref<{
    overall_percentage: number;
    completed_chapters: number;
    total_chapters: number;
} | null>(null);
const chapterProgresses = ref<Record<number, number>>({});

onBeforeMount(async () => {
    try {
        // Fetch content details and progress in parallel
        const api = await import('@/api/content');
        const progressApiModule = await import('@/api/progress');
        const { ApiClient } = await import('@/api/client');
        const { useAuthStore } = await import('@/stores/useAuthStore');
        const authStore = useAuthStore();

        const client = new ApiClient({
            baseUrl: import.meta.env.VITE_API_BASE_URL || '',
            getToken: () => authStore.token,
        });
        const contentApi = api.createContentApi(client);
        const progressApi = progressApiModule.createProgressApi(client);

        const [contentData, progressData] = await Promise.all([
            contentApi.get(contentId),
            progressApi.getContentProgress(contentId).catch(() => [])
        ]);

        // Update store with fetched content
        // This will update the computed 'content' property
        contentStore.selectContent(contentData);

        // Process progress data
        if (progressData && Array.isArray(progressData)) {
            // Sort by updated_at descending to find the last read
            const sortedProgress = [...progressData].sort((a, b) => 
                new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
            );
            
            if (sortedProgress.length > 0 && sortedProgress[0]) {
                lastReadChapterId.value = sortedProgress[0].chapter_id;
            }

            // Map progress
            const progressMap: Record<number, number> = {};
            let completedCount = 0;
            let totalProgressSum = 0;

            progressData.forEach(p => {
                progressMap[p.chapter_id] = p.percentage;
                if (p.percentage >= 100) completedCount++;
                totalProgressSum += p.percentage; // This is sum of chapter percentages
            });
            chapterProgresses.value = progressMap;

            // Calculate overall stats
            // We need total chapters count, which we get from contentData or when chapters are loaded
            // But contentData.chapter_count might be unreliable if scanning is pending
            // Let's use contentData.chapter_count as base
            const totalChapters = contentData.chapter_count;
            
            // Calculate weighted percentage
            // Each chapter contributes (1 / totalChapters) to the total
            // So sum(chapter_percentages) / totalChapters
            const overallPercentage = totalChapters > 0 
                ? Math.min(100, totalProgressSum / totalChapters)
                : 0;

            progressStats.value = {
                overall_percentage: overallPercentage,
                completed_chapters: completedCount,
                total_chapters: totalChapters
            };
        }

        // Load thumbnail
        if (contentData.has_thumbnail) {
            loadThumbnail(contentData.id);
        }

        // Load chapters list
        chaptersLoading.value = true;
        const chapterList = await contentApi.listChapters(contentId);
        chapters.value = chapterList.sort((a, b) => a.sort_order - b.sort_order);
        
        // Recalculate stats with actual chapter list count if different
        if (progressStats.value && chapters.value.length > 0 && chapters.value.length !== progressStats.value.total_chapters) {
             const totalChapters = chapters.value.length;
             let totalProgressSum = 0;
             Object.values(chapterProgresses.value).forEach(p => totalProgressSum += p);
             
             progressStats.value.total_chapters = totalChapters;
             progressStats.value.overall_percentage = totalChapters > 0 
                ? Math.min(100, totalProgressSum / totalChapters)
                : 0;
        }
        
        chaptersLoading.value = false;
    } catch (e) {
        console.error('Failed to fetch content:', e);
    } finally {
        contentLoading.value = false;
    }
});

// Extract information from metadata
const getMetaValue = (key: string): any => {
    const meta = content.value?.metadata;
    if (meta && typeof meta === 'object' && key in meta) {
        return (meta as Record<string, unknown>)[key] as string;
    }
    return null;
};

const getInfoboxValue = (key: string) => {
    const infobox = getMetaValue('infobox');
    if (Array.isArray(infobox)) {
        return infobox.find((item: any) => item.key === key)?.value;
    }
    return null;
};

const author = computed(() => {
    const infobox = getMetaValue('infobox')
    return infobox?.find((item: any) => item.key === '作者')?.value || infobox?.find((item: any) => item.key === '作画')?.value || infobox?.find((item: any) => item.key === '原作')?.value || t('content.author_unknown');
});

const publisher = computed(() => getMetaValue('publisher') || getInfoboxValue('出版社'));
const publishDate = computed(() => getMetaValue('publish_date') || getMetaValue('date') || getInfoboxValue('发售日'));
const isbn = computed(() => getMetaValue('isbn'));
const pageCount = computed(() => getMetaValue('page_count') || getMetaValue('pages'));
const description = computed(() => getMetaValue('description') || getMetaValue('summary'));
const rating = computed(() => {
    const r = getMetaValue('rating');
    return r?.score ?? null;
});
const originalName = computed(() => getMetaValue('name'));
const nameCn = computed(() => getMetaValue('name_cn') || getInfoboxValue('中文名'));
const platform = computed(() => getMetaValue('platform'));
const serializedIn = computed(() => getInfoboxValue('连载杂志'));
const director = computed(() => getInfoboxValue('导演'));
const characterDesign = computed(() => getInfoboxValue('人物设定'));
const music = computed(() => getInfoboxValue('音乐'));
const totalEpisodes = computed(() => {
    const eps = getMetaValue('eps');
    return eps && eps > 0 ? eps : null;
});
const totalVolumes = computed(() => {
    const vols = getMetaValue('volumes');
    return vols && vols > 0 ? vols : null;
});

const tags = computed(() => {
    const t = getMetaValue('tags');
    if (Array.isArray(t)) return t as unknown as {
        count: number;
        name: string;
        total_cont: number;
    }[];
});

// Render star rating
const renderStars = (score: number) => {
    const fullStars = Math.floor(score);
    const hasHalf = score - fullStars >= 0.5;
    return { fullStars, hasHalf, emptyStars: 10 - fullStars - (hasHalf ? 1 : 0) };
};

const handleStartReading = (chapterId?: number) => {
    if (chapterId) {
        router.push(`/read/${libraryId}/${contentId}/${chapterId}`);
    } else if (lastReadChapterId.value) {
        router.push(`/read/${libraryId}/${contentId}/${lastReadChapterId.value}`);
    } else if (chapters.value.length > 0) {
        // Default to first chapter
        router.push(`/read/${libraryId}/${contentId}/${chapters.value[0]!.id}`);
    } else {
        toast.error(t('content.no_chapters_toast'));
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
                    {{ lastReadChapterId ? t('content.continue_reading') : t('content.start_reading') }}
                </Button>

                <!-- Reading Progress -->
                <div v-if="progressStats" class="mt-4 space-y-2">
                    <div class="flex justify-between text-sm text-muted-foreground">
                        <span>{{ t('content.reading_progress') }}</span>
                        <span>{{ progressStats.overall_percentage.toFixed(0) }}%</span>
                    </div>
                    <Progress :model-value="progressStats.overall_percentage" class="h-2" />
                    <div class="text-xs text-muted-foreground text-center">
                        {{ t('content.chapters_read', {
                            completed: progressStats.completed_chapters, total:
                                progressStats.total_chapters
                        }) }}
                    </div>
                </div>
            </div>

            <!-- Right: Content Info -->
            <div class="flex-1 min-w-0">
                <!-- Title -->
                <h1 class="text-3xl font-bold text-foreground mb-3">
                    {{ content.title }}
                </h1>
                <h2 v-if="nameCn && nameCn !== content.title" class="text-xl text-foreground/80 mb-2">
                    {{ nameCn }}
                </h2>
                <h2 v-if="originalName && originalName !== content.title && originalName !== nameCn"
                    class="text-lg text-muted-foreground mb-3">
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
                    <!-- Platform -->
                    <div v-if="platform" class="flex flex-col gap-1 p-4 rounded-lg bg-muted/50">
                        <div class="flex items-center gap-2 text-xs text-muted-foreground">
                            <Tv class="size-4" />
                            <span>{{ t('content.platform') }}</span>
                        </div>
                        <span class="font-medium">{{ platform }}</span>
                    </div>

                    <!-- Publisher -->
                    <div v-if="publisher" class="flex flex-col gap-1 p-4 rounded-lg bg-muted/50">
                        <div class="flex items-center gap-2 text-xs text-muted-foreground">
                            <Building2 class="size-4" />
                            <span>{{ t('content.publisher') }}</span>
                        </div>
                        <span class="font-medium">{{ publisher }}</span>
                    </div>

                    <!-- Serialized In -->
                    <div v-if="serializedIn" class="flex flex-col gap-1 p-4 rounded-lg bg-muted/50">
                        <div class="flex items-center gap-2 text-xs text-muted-foreground">
                            <BookOpen class="size-4" />
                            <span>{{ t('content.serialized_in') }}</span>
                        </div>
                        <span class="font-medium">{{ serializedIn }}</span>
                    </div>

                    <!-- Director -->
                    <div v-if="director" class="flex flex-col gap-1 p-4 rounded-lg bg-muted/50">
                        <div class="flex items-center gap-2 text-xs text-muted-foreground">
                            <Clapperboard class="size-4" />
                            <span>{{ t('content.director') }}</span>
                        </div>
                        <span class="font-medium">{{ director }}</span>
                    </div>

                    <!-- Character Design -->
                    <div v-if="characterDesign" class="flex flex-col gap-1 p-4 rounded-lg bg-muted/50">
                        <div class="flex items-center gap-2 text-xs text-muted-foreground">
                            <Palette class="size-4" />
                            <span>{{ t('content.character_design') }}</span>
                        </div>
                        <span class="font-medium">{{ characterDesign }}</span>
                    </div>

                    <!-- Music -->
                    <div v-if="music" class="flex flex-col gap-1 p-4 rounded-lg bg-muted/50">
                        <div class="flex items-center gap-2 text-xs text-muted-foreground">
                            <Music class="size-4" />
                            <span>{{ t('content.music') }}</span>
                        </div>
                        <span class="font-medium">{{ music }}</span>
                    </div>

                    <!-- Publish Date -->
                    <div v-if="publishDate" class="flex flex-col gap-1 p-4 rounded-lg bg-muted/50">
                        <div class="flex items-center gap-2 text-xs text-muted-foreground">
                            <Calendar class="size-4" />
                            <span>{{ t('content.publish_date') }}</span>
                        </div>
                        <span class="font-medium">{{ publishDate }}</span>
                    </div>

                    <!-- ISBN -->
                    <div v-if="isbn" class="flex flex-col gap-1 p-4 rounded-lg bg-muted/50">
                        <div class="flex items-center gap-2 text-xs text-muted-foreground">
                            <Hash class="size-4" />
                            <span>{{ t('content.isbn') }}</span>
                        </div>
                        <span class="font-medium">{{ isbn }}</span>
                    </div>

                    <!-- Page Count -->
                    <div v-if="pageCount" class="flex flex-col gap-1 p-4 rounded-lg bg-muted/50">
                        <div class="flex items-center gap-2 text-xs text-muted-foreground">
                            <FileText class="size-4" />
                            <span>{{ t('content.page_count') }}</span>
                        </div>
                        <span class="font-medium">{{ pageCount }} {{ t('content.pages') }}</span>
                    </div>

                    <!-- Episodes -->
                    <div v-if="totalEpisodes" class="flex flex-col gap-1 p-4 rounded-lg bg-muted/50">
                        <div class="flex items-center gap-2 text-xs text-muted-foreground">
                            <Tv class="size-4" />
                            <span>{{ t('content.episodes') }}</span>
                        </div>
                        <span class="font-medium">{{ totalEpisodes }} {{ t('content.eps') }}</span>
                    </div>

                    <!-- Volumes -->
                    <div v-if="totalVolumes" class="flex flex-col gap-1 p-4 rounded-lg bg-muted/50">
                        <div class="flex items-center gap-2 text-xs text-muted-foreground">
                            <Layers class="size-4" />
                            <span>{{ t('content.volumes') }}</span>
                        </div>
                        <span class="font-medium">{{ totalVolumes }} {{ t('content.vols') }}</span>
                    </div>

                    <!-- Chapter Count (fallback) -->
                    <div v-if="!pageCount && !totalEpisodes && !totalVolumes"
                        class="flex flex-col gap-1 p-4 rounded-lg bg-muted/50">
                        <div class="flex items-center gap-2 text-xs text-muted-foreground">
                            <FileText class="size-4" />
                            <span>{{ t('content.chapter_count') }}</span>
                        </div>
                        <span class="font-medium">{{ content.chapter_count }} {{ t('content.chapters_unit') }}</span>
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
                        <h2 class="text-xl font-semibold">{{ t('content.chapters_title') }}</h2>
                        <span class="text-sm text-muted-foreground">{{ t('content.chapters_count', {
                            count:
                                chapters.length
                        }) }}</span>
                    </div>

                    <!-- Chapters Loading -->
                    <div v-if="chaptersLoading" class="space-y-2">
                        <Skeleton v-for="i in 5" :key="i" class="h-12 w-full rounded-lg" />
                    </div>

                    <!-- Chapters List -->
                    <div v-else-if="chapters.length > 0" class="space-y-2 overflow-y-auto pr-2">
                        <div v-for="chapter in chapters" :key="chapter.id"
                            class="flex items-center justify-between p-3 rounded-lg hover:bg-muted/60 transition-colors cursor-pointer group"
                            :class="lastReadChapterId === chapter.id ? 'bg-primary/10 hover:bg-primary/20' : 'bg-muted/30'"
                            @click="handleStartReading(chapter.id)">
                            <div class="flex items-center gap-3 min-w-0">
                                <span class="text-sm w-8 shrink-0"
                                    :class="lastReadChapterId === chapter.id ? 'text-primary font-medium' : 'text-muted-foreground'">
                                    {{ chapter.sort_order + 1 }}
                                </span>
                                <span class="truncate"
                                    :class="lastReadChapterId === chapter.id ? 'text-primary font-medium' : ''">
                                    {{ chapter.title }}
                                </span>
                            </div>
                            <div class="flex items-center gap-2">
                                <span v-if="chapterProgresses[chapter.id] !== undefined"
                                    class="text-xs text-muted-foreground mr-2">
                                    {{ chapterProgresses[chapter.id]?.toFixed(0) }}%
                                </span>
                                <span v-if="lastReadChapterId === chapter.id"
                                    class="text-xs text-primary font-medium px-2 py-0.5 rounded-full bg-primary/10">
                                    {{ t('content.last_read') }}
                                </span>
                                <ChevronRight
                                    class="size-4 text-muted-foreground group-hover:text-foreground transition-colors shrink-0" />
                            </div>
                        </div>
                    </div>

                    <!-- No Chapters -->
                    <div v-else class="text-center py-8 text-muted-foreground">
                        <p>{{ t('content.no_chapters') }}</p>
                    </div>
                </div>
            </div>
        </div>

        <!-- Error State -->
        <div v-else class="flex flex-col items-center justify-center py-20 text-muted-foreground">
            <p class="text-lg">{{ t('content.not_found') }}</p>
            <Button variant="outline" class="mt-4" @click="router.back()">
                {{ t('content.go_back') }}
            </Button>
        </div>
    </div>
</template>
