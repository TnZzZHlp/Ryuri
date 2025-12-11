<script setup lang="ts">
import { onBeforeMount } from 'vue';
import { useRouter } from 'vue-router';
import { useLibraryStore } from '@/stores/useLibraryStore';

const router = useRouter();
const library_id = router.currentRoute.value.params.id;
const libraryStore = useLibraryStore();

</script>

<template>
    <div>
        <h1>Library {{ library_id }}</h1>
        <div v-for="book in libraryStore." :key="book.id">
            <h2>{{ book.title }}</h2>
            <p>{{ book.author }}</p>
        </div>
    </div>
</template>