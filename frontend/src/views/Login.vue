<script setup lang="ts">
import { toTypedSchema } from '@vee-validate/zod'
import * as z from 'zod'
import { Button } from '@/components/ui/button'
import { useAuthStore } from '@/stores/useAuthStore'
import { toast } from 'vue-sonner'
import { useRouter } from 'vue-router'
import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
} from '@/components/ui/card'
import {
    Form,
    FormControl,
    FormField,
    FormItem,
    FormLabel,
    FormMessage,
} from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { Spinner } from '@/components/ui/spinner'
import type { ApiError } from '@/api'

const router = useRouter()
const authStore = useAuthStore()

const formSchema = toTypedSchema(
    z.object({
        username: z.string().min(1, 'Please enter your username'),
        password: z.string().min(1, 'Please enter your password'),
    }),
)

function onSubmit(values: Record<string, unknown>) {
    const { username, password } = values

    authStore.login(username as string, password as string)
        .then(() => {
            router.push('/')
        })
        .catch((error: ApiError) => {
            toast.error(error.message)
        })
}
</script>

<template>
    <div class="flex min-h-svh flex-col items-center justify-center bg-background p-6 md:p-10">
        <div class="flex w-full max-w-sm flex-col gap-6">
            <!-- Login Card -->
            <Card class="border-border/40 bg-card">
                <CardHeader class="text-center">
                    <CardTitle class="text-xl">Ryuri</CardTitle>
                </CardHeader>
                <CardContent>
                    <Form :validation-schema="formSchema" class="space-y-4" @submit="onSubmit">
                        <!-- Username Field -->
                        <FormField v-slot="{ componentField }" name="username">
                            <FormItem>
                                <FormLabel>Username</FormLabel>
                                <FormControl>
                                    <Input type="text" v-bind="componentField" autocomplete="username" />
                                </FormControl>
                                <FormMessage />
                            </FormItem>
                        </FormField>

                        <!-- Password Field -->
                        <FormField v-slot="{ componentField }" name="password">
                            <FormItem>
                                <div class="flex items-center justify-between">
                                    <FormLabel>Password</FormLabel>
                                </div>
                                <FormControl>
                                    <Input type="password" v-bind="componentField" autocomplete="current-password" />
                                </FormControl>
                                <FormMessage />
                            </FormItem>
                        </FormField>

                        <!-- Login Button -->
                        <Button type="submit" class="w-full mt-4" :disable="authStore.loading">
                            <Spinner v-if="authStore.loading" />Login
                        </Button>
                    </Form>
                </CardContent>
            </Card>
        </div>
    </div>
</template>
