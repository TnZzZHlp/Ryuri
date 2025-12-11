<script setup lang="ts">
import { toTypedSchema } from '@vee-validate/zod'
import * as z from 'zod'
import { Button } from '@/components/ui/button'
import { useAuth } from '@/composables/useAuth'
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
import type { ApiError } from '@/api'

const router = useRouter()

const formSchema = toTypedSchema(
    z.object({
        username: z.string().min(1, '请输入用户名'),
        password: z.string().min(1, '请输入密码'),
    }),
)

function onSubmit(values: Record<string, unknown>) {
    // 调用 API 登录
    const { login } = useAuth()

    toast.promise(login(values.username as string, values.password as string), {
        loading: '登录中...',
        success: () => {
            router.push('/dashboard')
        },
        error: (err: ApiError) => {
            console.debug(err)
            return err.message
        },
    })
}
</script>

<template>
    <div class="dark flex min-h-svh flex-col items-center justify-center bg-background p-6 md:p-10">
        <div class="flex w-full max-w-sm flex-col gap-6">
            <!-- Login Card -->
            <Card class="border-border/40 bg-card">
                <CardHeader class="text-center">
                    <CardTitle class="text-xl">欢迎回来</CardTitle>
                </CardHeader>
                <CardContent>
                    <Form :validation-schema="formSchema" class="space-y-4" @submit="onSubmit">
                        <!-- Username Field -->
                        <FormField v-slot="{ componentField }" name="username">
                            <FormItem>
                                <FormLabel>用户名</FormLabel>
                                <FormControl>
                                    <Input type="text" v-bind="componentField" />
                                </FormControl>
                                <FormMessage />
                            </FormItem>
                        </FormField>

                        <!-- Password Field -->
                        <FormField v-slot="{ componentField }" name="password">
                            <FormItem>
                                <div class="flex items-center justify-between">
                                    <FormLabel>密码</FormLabel>
                                    <a href="#"
                                        class="text-sm text-muted-foreground underline-offset-4 hover:underline">
                                        Forgot your password?
                                    </a>
                                </div>
                                <FormControl>
                                    <Input type="password" v-bind="componentField" />
                                </FormControl>
                                <FormMessage />
                            </FormItem>
                        </FormField>

                        <!-- Login Button -->
                        <Button type="submit" class="w-full mt-4">
                            登录
                        </Button>
                    </Form>
                </CardContent>
            </Card>

            <!-- Terms -->
            <p class="text-center text-xs text-muted-foreground">
                By clicking continue, you agree to our
                <a href="#" class="underline underline-offset-4 hover:text-foreground">Terms of Service</a>
                and
                <a href="#" class="underline underline-offset-4 hover:text-foreground">Privacy Policy</a>.
            </p>
        </div>
    </div>
</template>
