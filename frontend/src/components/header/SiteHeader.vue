<script setup lang="ts">
import { SidebarTrigger } from "@/components/ui/sidebar"
import { Button } from "@/components/ui/button"
import { Sun, ArrowLeft, MoreVertical, Search, Loader2 } from "lucide-vue-next";
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { toast } from 'vue-sonner';
import type { BangumiApi } from '@/api/bangumi';
import type { ContentApi } from '@/api/content';
import { useContentStore } from '@/stores/useContentStore';

const router = useRouter();

// change app theme
const changeAppTheme = () => {
    const html = document.querySelector("html")
    if (html) {
        html.classList.toggle("dark")
    }
}

const is_mobile = computed(() => {
    return window.innerWidth < 768
})

// Match Bangumi Feature
const showSearchDialog = ref(false);
const searchQuery = ref('');
const searchResults = ref<any[]>([]);
const searching = ref(false);
const searchFinished = ref(false);

let contentApi: ContentApi | null = null;
let bangumiApi: BangumiApi | null = null;

const initApis = async () => {
    if (contentApi && bangumiApi) return;
    const api = await import('@/api/content');
    const bangumiApiModule = await import('@/api/bangumi');
    const { ApiClient } = await import('@/api/client');
    const { useAuthStore } = await import('@/stores/useAuthStore');
    const authStore = useAuthStore();

    const client = new ApiClient({
        baseUrl: import.meta.env.VITE_API_BASE_URL || '',
        getToken: () => authStore.token,
    });
    contentApi = api.createContentApi(client);
    bangumiApi = bangumiApiModule.createBangumiApi(client);
};

const openSearchDialog = async () => {
    await initApis();
    showSearchDialog.value = true;
};

const performSearch = async () => {
    if (!searchQuery.value.trim() || !bangumiApi) return;

    searchFinished.value = false;

    searching.value = true;
    try {
        searchResults.value = await bangumiApi.searchSubjects({
            keyword: searchQuery.value,
            filter: {
                type: [1, 2], // Book and Anime
            }
        });
    } catch (e) {
        toast.error('Failed to search Bangumi');
        console.error(e);
    } finally {
        searching.value = false;
        searchFinished.value = true;
    }
};

const handleSelectResult = async (item: any) => {
    if (!contentApi) return;
    const contentId = Number(router.currentRoute.value.params.contentId);
    if (!contentId) return;

    try {
        await contentApi.updateMetadata(contentId, item);
        showSearchDialog.value = false;
        toast.success('Metadata updated successfully');

        // Refresh content
        const contentStore = useContentStore();
        contentStore.invalidateThumbnailCache(contentId);
        // Force reload to reflect changes
        window.location.reload();
    } catch (e) {
        toast.error('Failed to update metadata');
        console.error(e);
    }
};

</script>

<template>
    <header
        class="flex h-(--header-height) shrink-0 items-center gap-2 border-b text-foreground transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-(--header-height)">
        <div class="flex w-full items-center justify-between gap-1 px-4 lg:gap-2 lg:px-6">
            <div class="flex items-center">
                <SidebarTrigger class="-ml-1" v-if="is_mobile" />
                <ArrowLeft @click="router.push(`/library/${router.currentRoute.value.params.libraryId}`)"
                    v-if="router.currentRoute.value.name == 'Content'" class="cursor-pointer" :size=20 />
            </div>

            <div class="flex items-center gap-2">
                <DropdownMenu v-if="router.currentRoute.value.name === 'Content'">
                    <DropdownMenuTrigger as-child>
                        <Button variant="ghost" size="icon">
                            <MoreVertical class="size-5" />
                        </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                        <DropdownMenuItem @click="openSearchDialog">
                            <Search class="mr-2 size-4" />
                            <span>Match Bangumi</span>
                        </DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>

                <Button size="icon" variant="ghost" class="hidden lg:flex" @click="changeAppTheme">
                    <Sun />
                </Button>
            </div>
        </div>
    </header>

    <Dialog v-model:open="showSearchDialog">
        <DialogContent class="sm:max-w-[600px]">
            <DialogHeader>
                <DialogTitle>Match Bangumi Subject</DialogTitle>
            </DialogHeader>
            <div class="flex gap-2 my-4">
                <Input v-model="searchQuery" placeholder="Search Bangumi..." @keyup.enter="performSearch" />
                <Button @click="performSearch" :disabled="searching">
                    <Loader2 v-if="searching" class="animate-spin" />
                    <Search v-else />
                </Button>
            </div>

            <div class="max-h-[60vh] overflow-y-auto space-y-2">
                <div v-for="item in searchResults" :key="item.id"
                    class="flex gap-4 p-3 rounded-lg border hover:bg-muted cursor-pointer transition-colors"
                    @click="handleSelectResult(item)">
                    <img v-if="item.images?.common || item.images?.medium"
                        :src="item.images.common || item.images.medium"
                        class="w-16 h-24 object-cover rounded shadow-sm" />
                    <div v-else class="w-16 h-24 bg-muted rounded flex items-center justify-center">
                        <span class="text-xs text-muted-foreground">No Img</span>
                    </div>
                    <div class="flex-1 min-w-0">
                        <div class="font-bold truncate">{{ item.name_cn || item.name }}</div>
                        <div class="text-sm text-muted-foreground">{{ item.date }}</div>
                        <div class="text-xs text-muted-foreground mt-1 line-clamp-2">{{ item.summary }}</div>
                    </div>
                </div>
                <div v-if="searchResults.length === 0 && !searching && searchQuery.length !== 0 && searchFinished"
                    class="text-center text-muted-foreground py-4">
                    No results found
                </div>
            </div>
        </DialogContent>
    </Dialog>
</template>
