<script setup lang="ts">
import { Progress } from '@/components/ui/progress'

interface Props {
    value: number
}

defineProps<Props>()
</script>

<template>
    <Progress :model-value="value" class="absolute top-0 left-0 right-0 h-1 rounded-none bg-white/20" />
</template>
