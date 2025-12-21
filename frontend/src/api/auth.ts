/**
 * Authentication API Module
 *
 * Provides functions for user authentication and profile management.
 *
 * **Implements: Requirements 2.1, 2.2, 2.3, 2.4**
 */

import { ApiClient } from "./client";
import type {
    LoginRequest,
    LoginResponse,
    UserResponse,
    UpdateUserRequest,
} from "./types";

/**
 * Authentication API interface.
 */
export interface AuthApi {
    login(username: string, password: string): Promise<LoginResponse>;
    getMe(): Promise<UserResponse>;
    updateMe(request: UpdateUserRequest): Promise<UserResponse>;
}

/**
 * Creates an Auth API instance using the provided API client.
 *
 * @param client - The API client to use for HTTP requests
 * @returns An AuthApi implementation
 */
export function createAuthApi(client: ApiClient): AuthApi {
    return {
        /**
         * Authenticates a user with username and password.
         * Returns user info and JWT token on success.
         *
         * **Implements: Requirement 2.1**
         *
         * @param username - The user's username
         * @param password - The user's password
         * @returns LoginResponse containing user info and token
         */
        async login(
            username: string,
            password: string
        ): Promise<LoginResponse> {
            const request: LoginRequest = { username, password };
            return client.post<LoginResponse>("/api/auth/login", request, {
                requiresAuth: false,
            });
        },

        /**
         * Retrieves the current authenticated user's profile.
         *
         * **Implements: Requirement 2.2**
         *
         * @returns The current user's information
         */
        async getMe(): Promise<UserResponse> {
            return client.get<UserResponse>("/api/auth/me");
        },

        /**
         * Updates the current authenticated user's profile.
         *
         * **Implements: Requirement 2.3**
         *
         * @param request - The update request containing fields to update
         * @returns The updated user information
         */
        async updateMe(request: UpdateUserRequest): Promise<UserResponse> {
            return client.put<UserResponse>("/api/auth/me", request);
        },
    };
}
