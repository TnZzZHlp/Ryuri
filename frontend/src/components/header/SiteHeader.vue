<script setup lang="ts">
import { SidebarTrigger } from "@/components/ui/sidebar"
import { Button } from "@/components/ui/button"
import { Sun, ArrowLeft, MoreVertical, Search, Loader2, Pencil, Save } from "lucide-vue-next";
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from 'vue-i18n';
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
import { Label } from '@/components/ui/label';
import { toast } from 'vue-sonner';
import type { BangumiApi } from '@/api/bangumi';
import type { ContentApi } from '@/api/content';
import { useContentStore } from '@/stores/useContentStore';
import { Codemirror } from 'vue-codemirror';
import { json } from '@codemirror/lang-json';
import { EditorView } from "@codemirror/view";

const router = useRouter();
const contentStore = useContentStore();
const { t } = useI18n();

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

// Edit Content Feature
const showEditDialog = ref(false);
const activeTab = ref<'general' | 'bangumi'>('general');
const currentTitle = ref('');
const currentMetadata = ref('');
const isSaving = ref(false);

const jsonError = computed(() => {
    if (!currentMetadata.value.trim()) return null;
    try {
        JSON.parse(currentMetadata.value);
        return null;
    } catch (e) {
        return e instanceof Error ? e.message : t('edit_content.invalid_json');
    }
});

const fontTheme = EditorView.theme({
    "&": {
        fontSize: "14px",
    },
    ".cm-content": {
        fontFamily: "'JetBrains Mono', monospace",
    }
}, { dark: true });

const extensions = [json(), EditorView.lineWrapping, fontTheme];

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

const openEditDialog = async () => {
    await initApis();
    const contentId = Number(router.currentRoute.value.params.contentId);
    if (!contentId || !contentApi) return;

    try {
        const content = await contentApi.get(contentId);
        currentTitle.value = content.title;
        currentMetadata.value = content.metadata ? JSON.stringify(content.metadata, null, 2) : '{}';
        showEditDialog.value = true;
        // Reset search state
        searchQuery.value = '';
        searchResults.value = [];
        searchFinished.value = false;
        activeTab.value = 'general';
    } catch (e) {
        toast.error(t('edit_content.load_fail'));
        console.error(e);
    }
};

const saveContent = async () => {
    if (!contentApi) return;
    if (jsonError.value) {
        toast.error(t('edit_content.invalid_json'));
        return;
    }
    const contentId = Number(router.currentRoute.value.params.contentId);

    let metadata: any = null;
    if (currentMetadata.value.trim()) {
        metadata = JSON.parse(currentMetadata.value);
    }

    isSaving.value = true;
    try {
        // Update content
        await contentApi.update(contentId, {
            title: currentTitle.value,
            metadata: metadata
        });
        toast.success(t('edit_content.update_success'));

        // Retrieve fresh content data
        const freshContent = await contentApi.get(contentId);
        contentStore.updateContentInStore(freshContent);

        // Refresh library list (to ensure correct sort order if title changed)
        const libraryId = Number(router.currentRoute.value.params.libraryId);
        if (libraryId) {
            contentStore.fetchContents(libraryId, true);
        }

        // Force refresh thumbnail if metadata changed
        if (metadata) {
            contentStore.invalidateThumbnailCache(contentId);
        }
    } catch (e) {
        toast.error(t('edit_content.update_fail'));
        console.error(e);
    } finally {
        isSaving.value = false;
        showEditDialog.value = false;
    }
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
        toast.error(t('edit_content.search_fail'));
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
        // Update local metadata
        await contentApi.update(contentId, { metadata: item });

        toast.success(t('edit_content.metadata_update_success'));

        // Retrieve fresh content data
        const freshContent = await contentApi.get(contentId);
        contentStore.updateContentInStore(freshContent);

        // Refresh library list
        const libraryId = Number(router.currentRoute.value.params.libraryId);
        if (libraryId) {
            contentStore.fetchContents(libraryId, true);
        }

        contentStore.invalidateThumbnailCache(contentId);

        showEditDialog.value = false;
    } catch (e) {
        toast.error(t('edit_content.metadata_update_fail'));
        console.error(e);
    }
};

</script>

<template>
    <header
        class="flex h-(--header-height) shrink-0 items-center gap-2 border-b text-foreground transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-(--header-height)">
        <div class="flex w-full items-center justify-between gap-1 px-4 lg:gap-2 lg:px-6">
            <div class="flex items-center gap-2">
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
                        <DropdownMenuItem @click="openEditDialog">
                            <Pencil class="mr-2 size-4" />
                            <span>{{ t('edit_content.title') }}</span>
                        </DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>

                <Button size="icon" variant="ghost" class="hidden lg:flex" @click="changeAppTheme">
                    <Sun />
                </Button>
            </div>
        </div>
    </header>

    <Dialog v-model:open="showEditDialog">
        <DialogContent class="sm:max-w-[800px]">
            <DialogHeader>
                <DialogTitle>{{ t('edit_content.title') }}</DialogTitle>
            </DialogHeader>

            <!-- Custom Tabs -->
            <div class="flex space-x-1 rounded-lg bg-muted p-1">
                <button @click="activeTab = 'general'" :class="[
                    'flex-1 flex items-center justify-center rounded-md px-3 py-1.5 text-sm font-medium transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
                    activeTab === 'general' ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:bg-background/50 hover:text-foreground'
                ]">
                    <Pencil class="mr-2 size-4" />
                    {{ t('edit_content.tab_general') }}
                </button>
                <button @click="activeTab = 'bangumi'" :class="[
                    'flex-1 flex items-center justify-center rounded-md px-3 py-1.5 text-sm font-medium transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
                    activeTab === 'bangumi' ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:bg-background/50 hover:text-foreground'
                ]">
                    <Search class="mr-2 size-4" />
                    {{ t('edit_content.tab_bangumi') }}
                </button>
            </div>

            <!-- General Tab Content -->
            <div v-if="activeTab === 'general'" class="py-4 space-y-4">
                <div class="space-y-2">
                    <Label for="title">{{ t('edit_content.title_label') }}</Label>
                    <Input id="title" v-model="currentTitle" :placeholder="t('edit_content.title_placeholder')" />
                </div>
                <div class="space-y-2">
                    <Label for="metadata">{{ t('edit_content.metadata_label') }}</Label>
                    <div class="border rounded-md overflow-hidden h-[300px]"
                        :class="{ 'border-destructive': !!jsonError }">
                        <Codemirror v-model="currentMetadata" :placeholder="t('edit_content.metadata_placeholder')"
                            :style="{ height: '100%' }" :autofocus="true" :indent-with-tab="true" :tab-size="2"
                            :extensions="extensions" />
                    </div>
                    <p class="text-xs text-muted-foreground" v-if="!jsonError">
                        {{ t('edit_content.metadata_warning') }}
                    </p>
                    <p class="text-xs text-destructive font-mono" v-else>
                        {{ jsonError }}
                    </p>
                </div>
                <div class="flex justify-end">
                    <Button @click="saveContent" :disabled="isSaving || !!jsonError">
                        <Loader2 v-if="isSaving" class="mr-2 size-4 animate-spin" />
                        <Save v-else class="mr-2 size-4" />
                        {{ t('edit_content.save_btn') }}
                    </Button>
                </div>
            </div>

            <!-- Bangumi Tab Content -->
            <div v-else class="space-y-4">
                <div class="flex gap-2 mt-2">
                    <Input v-model="searchQuery" :placeholder="t('edit_content.search_bangumi_placeholder')"
                        @keyup.enter="performSearch" />
                    <Button @click="performSearch" :disabled="searching">
                        <Loader2 v-if="searching" class="animate-spin" />
                        <Search v-else />
                    </Button>
                </div>

                <div class="max-h-[50vh] overflow-y-auto space-y-2">
                    <div v-for="item in searchResults" :key="item.id"
                        class="flex gap-4 p-3 rounded-lg border hover:bg-muted cursor-pointer transition-colors"
                        @click="handleSelectResult(item)">
                        <img v-if="item.images?.common || item.images?.medium"
                            :src="item.images.common || item.images.medium"
                            class="w-16 h-24 object-cover rounded shadow-sm" />
                        <div v-else class="w-16 h-24 bg-muted rounded flex items-center justify-center">
                            <span class="text-xs text-muted-foreground">{{ t('edit_content.no_img') }}</span>
                        </div>
                        <div class="flex-1 min-w-0">
                            <div class="font-bold truncate">{{ item.name_cn || item.name }}</div>
                            <div class="text-sm text-muted-foreground">{{ item.date }}</div>
                            <div class="text-xs text-muted-foreground mt-1 line-clamp-2">{{ item.summary }}</div>
                        </div>
                    </div>
                    <div v-if="searchResults.length === 0 && !searching && searchQuery.length !== 0 && searchFinished"
                        class="text-center text-muted-foreground py-4">
                        {{ t('edit_content.no_results') }}
                    </div>
                </div>
            </div>
        </DialogContent>
    </Dialog>
</template>
