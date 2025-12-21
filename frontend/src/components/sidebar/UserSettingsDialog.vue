<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { Button } from '@/components/ui/button'
import {
    DialogClose,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { useAuthStore } from '@/stores/useAuthStore'
import { useApiKeyStore } from '@/stores/useApiKeyStore'
import { toast } from 'vue-sonner'
import { Copy, Trash } from 'lucide-vue-next'
import type { UserResponse } from '@/api/types'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
    user: UserResponse
}>()

const emit = defineEmits<{
    (e: 'close'): void
}>()

const authStore = useAuthStore()
const apiKeyStore = useApiKeyStore()
const { t } = useI18n()
const appVersion = __APP_VERSION__

// Profile State
const username = ref(props.user.username || '')
const bangumiApiKey = ref(props.user.bangumi_api_key || '')
const profileLoading = ref(false)

// Password State
const oldPassword = ref('')
const newPassword = ref('')
const confirmPassword = ref('')
const passwordLoading = ref(false)

// API Key State
const newApiKeyName = ref('')
const apiKeyCreationLoading = ref(false)

// Watch for user changes to update local state
watch(() => props.user, (newUser) => {
    username.value = newUser.username || ''
    bangumiApiKey.value = newUser.bangumi_api_key || ''
}, { immediate: true })

onMounted(() => {
    apiKeyStore.fetchApiKeys()
})

async function handleUpdateProfile() {
    profileLoading.value = true
    try {
        await authStore.updateUser({
            bangumi_api_key: bangumiApiKey.value || undefined
        })
        toast.success(t('library.profile_updated'))
    } catch (e) {
        toast.error(e instanceof Error ? e.message : t('library.profile_update_fail'))
    } finally {
        profileLoading.value = false
    }
}

async function handleUpdateSecurity() {
    const payload: { username?: string; old_password?: string; password?: string } = {}
    let hasChanges = false

    // Username update
    if (username.value !== props.user.username) {
        payload.username = username.value
        hasChanges = true
    }

    // Password update
    const isPasswordChange = oldPassword.value || newPassword.value || confirmPassword.value
    if (isPasswordChange) {
        if (!oldPassword.value || !newPassword.value || !confirmPassword.value) {
            toast.error(t('library.password_fields_required'))
            return
        }

        if (newPassword.value !== confirmPassword.value) {
            toast.error(t('library.password_mismatch'))
            return
        }
        payload.old_password = oldPassword.value
        payload.password = newPassword.value
        hasChanges = true
    }

    if (!hasChanges) return

    passwordLoading.value = true
    try {
        await authStore.updateUser(payload)
        toast.success(t('library.profile_updated'))
        // Clear password fields if changed
        if (payload.password) {
            oldPassword.value = ''
            newPassword.value = ''
            confirmPassword.value = ''
        }
    } catch (e) {
        toast.error(e instanceof Error ? e.message : t('library.profile_update_fail'))
    } finally {
        passwordLoading.value = false
    }
}

async function handleCreateApiKey() {
    if (!newApiKeyName.value) return
    apiKeyCreationLoading.value = true
    try {
        await apiKeyStore.createApiKey({ name: newApiKeyName.value })
        newApiKeyName.value = ''
        toast.success(t('library.api_key_created'))
    } catch (e) {
        toast.error(e instanceof Error ? e.message : t('library.api_key_create_fail'))
    } finally {
        apiKeyCreationLoading.value = false
    }
}

async function handleDeleteApiKey(id: number) {
    if (!confirm(t('library.api_key_delete_confirm'))) return
    try {
        await apiKeyStore.deleteApiKey(id)
        toast.success(t('library.api_key_deleted'))
    } catch (e) {
        toast.error(e instanceof Error ? e.message : t('library.api_key_delete_fail'))
    }
}

function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text)
    toast.success(t('library.copied_to_clipboard'))
}
</script>

<template>
    <DialogContent class="sm:max-w-[500px] max-h-[85vh] overflow-y-auto">
        <DialogHeader>
            <DialogTitle>{{ t('library.user_settings_title') }}</DialogTitle>
            <DialogDescription>
                {{ t('library.user_settings_desc') }}
            </DialogDescription>
        </DialogHeader>

        <div class="grid gap-6 py-4">
            <!-- Profile Section -->
            <div class="space-y-4">
                <h3 class="text-lg font-medium">{{ t('library.profile_section') }}</h3>
                <div class="grid gap-2">
                    <Label for="bangumi-key">{{ t('library.bangumi_key_label') }}</Label>
                    <Input id="bangumi-key" v-model="bangumiApiKey" :placeholder="t('library.bangumi_key_placeholder')" />
                    <p class="text-xs text-muted-foreground">
                        {{ t('library.bangumi_key_help') }}
                    </p>
                </div>
                <div class="flex justify-end">
                    <Button @click="handleUpdateProfile" :disabled="profileLoading" size="sm">
                        {{ profileLoading ? t('library.saving_btn') : t('library.save_profile_btn') }}
                    </Button>
                </div>
            </div>

            <Separator />

            <!-- Security Section -->
            <div class="space-y-4">
                <h3 class="text-lg font-medium">{{ t('library.security_section') }}</h3>
                <div class="grid gap-2">
                    <Label for="username">{{ t('login.username') }}</Label>
                    <Input id="username" v-model="username" />
                </div>
                <div class="grid gap-2">
                    <Label for="old-password">{{ t('library.old_password_label') }}</Label>
                    <Input id="old-password" type="password" autocomplete="current-password" v-model="oldPassword" />
                </div>
                <div class="grid gap-2">
                    <Label for="new-password">{{ t('library.new_password_label') }}</Label>
                    <Input id="new-password" type="password" autocomplete="new-password" v-model="newPassword" />
                </div>
                <div class="grid gap-2">
                    <Label for="confirm-password">{{ t('library.confirm_password_label') }}</Label>
                    <Input id="confirm-password" type="password" autocomplete="new-password"
                        v-model="confirmPassword" />
                </div>
                <div class="flex justify-end">
                    <Button @click="handleUpdateSecurity" :disabled="passwordLoading" size="sm">
                        {{ passwordLoading ? t('library.saving_btn') : t('library.save_btn') }}
                    </Button>
                </div>
            </div>

            <Separator />

            <!-- API Keys Section -->
            <div class="space-y-4">
                <h3 class="text-lg font-medium">{{ t('library.api_keys_section') }}</h3>
                <div class="grid gap-4">
                    <div class="flex gap-2">
                        <Input v-model="newApiKeyName" :placeholder="t('library.api_key_name_placeholder')"
                            @keyup.enter="handleCreateApiKey" />
                        <Button @click="handleCreateApiKey" :disabled="apiKeyCreationLoading || !newApiKeyName">
                            {{ t('library.create_api_key_btn') }}
                        </Button>
                    </div>

                    <div class="space-y-2">
                        <div v-for="key in apiKeyStore.apiKeys" :key="key.id"
                            class="flex items-center justify-between p-3 border rounded-md">
                            <div class="grid gap-1 overflow-hidden">
                                <p class="font-medium text-sm truncate">{{ key.name }}</p>
                                <div class="flex items-center gap-2">
                                    <code
                                        class="text-xs bg-muted px-1 py-0.5 rounded truncate max-w-[200px]">{{ key.api_key }}</code>
                                    <Button variant="ghost" size="icon" class="h-6 w-6 shrink-0"
                                        @click="copyToClipboard(key.api_key)">
                                        <Copy class="h-3 w-3" />
                                    </Button>
                                </div>
                                <p class="text-xs text-muted-foreground">Created: {{ new
                                    Date(key.created_at).toLocaleDateString() }}</p>
                            </div>
                            <Button variant="ghost" size="icon"
                                class="text-destructive hover:text-destructive hover:bg-destructive/10 shrink-0"
                                @click="handleDeleteApiKey(key.id)">
                                <Trash class="h-4 w-4" />
                            </Button>
                        </div>
                        <p v-if="apiKeyStore.apiKeys.length === 0" class="text-sm text-muted-foreground text-center py-2">
                            {{ t('library.no_api_keys') }}
                        </p>
                    </div>
                </div>
            </div>
        </div>

        <DialogFooter class="sm:justify-between gap-2">
            <div class="flex items-center text-xs text-muted-foreground">
                Ryuri v{{ appVersion }}
            </div>
            <DialogClose as-child>
                <Button variant="outline" @click="emit('close')">
                    {{ t('library.close_btn') }}
                </Button>
            </DialogClose>
        </DialogFooter>
    </DialogContent>
</template>
