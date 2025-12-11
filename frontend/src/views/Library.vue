<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useContentStore } from '@/stores/useContentStore';
import { onBeforeMount } from 'vue';

const router = useRouter();
const library_id: number = router.currentRoute.value.params.id as unknown as number;
const { contents, fetchContents } = useContentStore();

onBeforeMount(() => {
    if (!contents.get(library_id)) {
        fetchContents(library_id);
    }
})

</script>

<template>
    <div>
        <h1>Library {{ library_id }}</h1>
        <div v-for="book in contents.get(library_id)" :key="book.id">
            <h2>{{ book.title }}</h2>
        </div>
    </div>
</template>