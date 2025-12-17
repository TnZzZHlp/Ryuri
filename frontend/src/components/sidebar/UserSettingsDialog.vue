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

const props = defineProps<{
    user: UserResponse
}>()

const emit = defineEmits<{
    (e: 'close'): void
}>()

const authStore = useAuthStore()
const apiKeyStore = useApiKeyStore()

// Profile State
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
        toast.success('Profile updated successfully')
    } catch (e) {
        toast.error(e instanceof Error ? e.message : 'Failed to update profile')
    } finally {
        profileLoading.value = false
    }
}

async function handleUpdatePassword() {
    if (!oldPassword.value || !newPassword.value || !confirmPassword.value) {
        toast.error('Please fill in all password fields')
        return
    }

    if (newPassword.value !== confirmPassword.value) {
        toast.error('New passwords do not match')
        return
    }

    passwordLoading.value = true
    try {
        await authStore.updatePassword({
            old_password: oldPassword.value,
            new_password: newPassword.value
        })
        toast.success('Password changed successfully')
        // Clear password fields
        oldPassword.value = ''
        newPassword.value = ''
        confirmPassword.value = ''
    } catch (e) {
        toast.error(e instanceof Error ? e.message : 'Failed to change password')
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
        toast.success('API Key created successfully')
    } catch (e) {
        toast.error(e instanceof Error ? e.message : 'Failed to create API key')
    } finally {
        apiKeyCreationLoading.value = false
    }
}

async function handleDeleteApiKey(id: number) {
    if (!confirm('Are you sure you want to delete this API key? This action cannot be undone.')) return
    try {
        await apiKeyStore.deleteApiKey(id)
        toast.success('API Key deleted successfully')
    } catch (e) {
        toast.error(e instanceof Error ? e.message : 'Failed to delete API key')
    }
}

function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text)
    toast.success('Copied to clipboard')
}
</script>

<template>
    <DialogContent class="sm:max-w-[500px] max-h-[85vh] overflow-y-auto">
        <DialogHeader>
            <DialogTitle>User Settings</DialogTitle>
            <DialogDescription>
                Manage your profile and security settings
            </DialogDescription>
        </DialogHeader>

        <div class="grid gap-6 py-4">
            <!-- Profile Section -->
            <div class="space-y-4">
                <h3 class="text-lg font-medium">Profile</h3>
                <div class="grid gap-2">
                    <Label for="bangumi-key">Bangumi API Key</Label>
                    <Input id="bangumi-key" v-model="bangumiApiKey" placeholder="Enter your Bangumi API Key" />
                    <p class="text-xs text-muted-foreground">
                        Used for fetching metadata from Bangumi.
                    </p>
                </div>
                <div class="flex justify-end">
                    <Button @click="handleUpdateProfile" :disabled="profileLoading" size="sm">
                        {{ profileLoading ? 'Saving...' : 'Save Profile' }}
                    </Button>
                </div>
            </div>

            <Separator />

            <!-- Security Section -->
            <div class="space-y-4">
                <h3 class="text-lg font-medium">Security</h3>
                <div class="grid gap-2">
                    <Label for="old-password">Current Password</Label>
                    <Input id="old-password" type="password" autocomplete="current-password" v-model="oldPassword" />
                </div>
                <div class="grid gap-2">
                    <Label for="new-password">New Password</Label>
                    <Input id="new-password" type="password" autocomplete="new-password" v-model="newPassword" />
                </div>
                <div class="grid gap-2">
                    <Label for="confirm-password">Confirm Password</Label>
                    <Input id="confirm-password" type="password" autocomplete="new-password"
                        v-model="confirmPassword" />
                </div>
                <div class="flex justify-end">
                    <Button @click="handleUpdatePassword" :disabled="passwordLoading" size="sm" variant="destructive">
                        {{ passwordLoading ? 'Changing...' : 'Change Password' }}
                    </Button>
                </div>
            </div>

            <Separator />

            <!-- API Keys Section -->
            <div class="space-y-4">
                <h3 class="text-lg font-medium">API Keys</h3>
                <div class="grid gap-4">
                    <div class="flex gap-2">
                        <Input v-model="newApiKeyName" placeholder="Key Name (e.g. Mobile App)"
                            @keyup.enter="handleCreateApiKey" />
                        <Button @click="handleCreateApiKey" :disabled="apiKeyCreationLoading || !newApiKeyName">
                            Create
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
                            No API keys found.
                        </p>
                    </div>
                </div>
            </div>
        </div>

        <DialogFooter>
            <DialogClose as-child>
                <Button variant="outline" @click="emit('close')">
                    Close
                </Button>
            </DialogClose>
        </DialogFooter>
    </DialogContent>
</template>
