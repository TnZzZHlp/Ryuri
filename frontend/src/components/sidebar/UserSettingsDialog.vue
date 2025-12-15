<script setup lang="ts">
import { ref, watch } from 'vue'
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
import { toast } from 'vue-sonner'
import type { UserResponse } from '@/api/types'

const props = defineProps<{
    user: UserResponse
}>()

const emit = defineEmits<{
    (e: 'close'): void
}>()

const authStore = useAuthStore()

// Profile State
const bangumiApiKey = ref(props.user.bangumi_api_key || '')
const profileLoading = ref(false)

// Password State
const oldPassword = ref('')
const newPassword = ref('')
const confirmPassword = ref('')
const passwordLoading = ref(false)

// Watch for user changes to update local state
watch(() => props.user, (newUser) => {
    bangumiApiKey.value = newUser.bangumi_api_key || ''
}, { immediate: true })

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
</script>

<template>
    <DialogContent class="sm:max-w-[500px]">
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
