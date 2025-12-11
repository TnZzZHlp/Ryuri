/**
 * Unit tests for Auth API module.
 *
 * Tests authentication API functions including login, profile retrieval,
 * profile update, and password change.
 *
 * **Validates: Requirements 2.1, 2.2, 2.3, 2.4**
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { ApiClient } from "../client";
import { createAuthApi } from "../auth";
import { ApiError } from "../types";
import type {
    LoginResponse,
    UserResponse,
    UpdateUserRequest,
    UpdatePasswordRequest,
} from "../types";

describe("Auth API", () => {
    let mockFetch: ReturnType<typeof vi.fn>;

    beforeEach(() => {
        mockFetch = vi.fn();
        vi.stubGlobal("fetch", mockFetch);
    });

    afterEach(() => {
        vi.unstubAllGlobals();
    });

    /**
     * Helper to create a mock successful response.
     */
    function mockSuccessResponse<T>(data: T) {
        return {
            ok: true,
            status: 200,
            headers: new Headers({ "content-length": "100" }),
            json: async () => data,
        };
    }

    /**
     * Helper to create a mock error response.
     */
    function mockErrorResponse(status: number, message: string) {
        return {
            ok: false,
            status,
            statusText: "Error",
            text: async () => JSON.stringify({ error: message }),
        };
    }

    describe("login", () => {
        /**
         * **Validates: Requirement 2.1**
         * WHEN a user provides valid credentials, THE Auth_API SHALL return
         * a LoginResponse containing user info and JWT token
         */

        it("sends login request with correct credentials", async () => {
            const mockResponse: LoginResponse = {
                user: {
                    id: 1,
                    username: "testuser",
                    bangumi_api_key: null,
                    created_at: "2024-01-01T00:00:00Z",
                },
                token: "jwt.token.here",
            };
            mockFetch.mockResolvedValue(mockSuccessResponse(mockResponse));

            const client = new ApiClient({ baseUrl: "http://localhost:3000" });
            const authApi = createAuthApi(client);

            const result = await authApi.login("testuser", "password123");

            expect(result).toEqual(mockResponse);
            expect(mockFetch).toHaveBeenCalledTimes(1);

            const [url, init] = mockFetch.mock.calls[0] as [
                string,
                RequestInit
            ];
            expect(url).toBe("http://localhost:3000/api/auth/login");
            expect(init.method).toBe("POST");
            expect(JSON.parse(init.body as string)).toEqual({
                username: "testuser",
                password: "password123",
            });
        });

        it("does not include auth header for login request", async () => {
            const mockResponse: LoginResponse = {
                user: {
                    id: 1,
                    username: "testuser",
                    bangumi_api_key: null,
                    created_at: "2024-01-01T00:00:00Z",
                },
                token: "jwt.token.here",
            };
            mockFetch.mockResolvedValue(mockSuccessResponse(mockResponse));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "existing.token",
            });
            const authApi = createAuthApi(client);

            await authApi.login("testuser", "password123");

            const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
            const headers = init.headers as Record<string, string>;
            expect(headers["Authorization"]).toBeUndefined();
        });

        it("throws ApiError on invalid credentials", async () => {
            mockFetch.mockResolvedValue(
                mockErrorResponse(401, "Invalid credentials")
            );

            const client = new ApiClient({ baseUrl: "http://localhost:3000" });
            const authApi = createAuthApi(client);

            await expect(authApi.login("wrong", "wrong")).rejects.toThrow(
                ApiError
            );

            try {
                await authApi.login("wrong", "wrong");
            } catch (error) {
                expect(error).toBeInstanceOf(ApiError);
                expect((error as ApiError).status).toBe(401);
            }
        });
    });

    describe("getMe", () => {
        /**
         * **Validates: Requirement 2.2**
         * WHEN an authenticated user requests their profile, THE Auth_API SHALL
         * return the current User information
         */

        it("retrieves current user profile with auth header", async () => {
            const mockUser: UserResponse = {
                id: 1,
                username: "testuser",
                bangumi_api_key: "api-key-123",
                created_at: "2024-01-01T00:00:00Z",
            };
            mockFetch.mockResolvedValue(mockSuccessResponse(mockUser));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.jwt.token",
            });
            const authApi = createAuthApi(client);

            const result = await authApi.getMe();

            expect(result).toEqual(mockUser);
            expect(mockFetch).toHaveBeenCalledTimes(1);

            const [url, init] = mockFetch.mock.calls[0] as [
                string,
                RequestInit
            ];
            expect(url).toBe("http://localhost:3000/api/auth/me");
            expect(init.method).toBe("GET");

            const headers = init.headers as Record<string, string>;
            expect(headers["Authorization"]).toBe("Bearer valid.jwt.token");
        });

        it("throws ApiError when not authenticated", async () => {
            mockFetch.mockResolvedValue(mockErrorResponse(401, "Unauthorized"));

            const client = new ApiClient({ baseUrl: "http://localhost:3000" });
            const authApi = createAuthApi(client);

            await expect(authApi.getMe()).rejects.toThrow(ApiError);

            try {
                await authApi.getMe();
            } catch (error) {
                expect(error).toBeInstanceOf(ApiError);
                expect((error as ApiError).isUnauthorized()).toBe(true);
            }
        });
    });

    describe("updateMe", () => {
        /**
         * **Validates: Requirement 2.3**
         * WHEN an authenticated user updates their profile, THE Auth_API SHALL
         * send the update request and return the updated User info
         */

        it("updates user profile and returns updated data", async () => {
            const updateRequest: UpdateUserRequest = {
                bangumi_api_key: "new-api-key",
            };
            const mockUpdatedUser: UserResponse = {
                id: 1,
                username: "testuser",
                bangumi_api_key: "new-api-key",
                created_at: "2024-01-01T00:00:00Z",
            };
            mockFetch.mockResolvedValue(mockSuccessResponse(mockUpdatedUser));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.jwt.token",
            });
            const authApi = createAuthApi(client);

            const result = await authApi.updateMe(updateRequest);

            expect(result).toEqual(mockUpdatedUser);
            expect(mockFetch).toHaveBeenCalledTimes(1);

            const [url, init] = mockFetch.mock.calls[0] as [
                string,
                RequestInit
            ];
            expect(url).toBe("http://localhost:3000/api/auth/me");
            expect(init.method).toBe("PUT");
            expect(JSON.parse(init.body as string)).toEqual(updateRequest);

            const headers = init.headers as Record<string, string>;
            expect(headers["Authorization"]).toBe("Bearer valid.jwt.token");
        });

        it("handles clearing bangumi_api_key", async () => {
            const updateRequest: UpdateUserRequest = {
                bangumi_api_key: null,
            };
            const mockUpdatedUser: UserResponse = {
                id: 1,
                username: "testuser",
                bangumi_api_key: null,
                created_at: "2024-01-01T00:00:00Z",
            };
            mockFetch.mockResolvedValue(mockSuccessResponse(mockUpdatedUser));

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.jwt.token",
            });
            const authApi = createAuthApi(client);

            const result = await authApi.updateMe(updateRequest);

            expect(result.bangumi_api_key).toBeNull();
        });

        it("throws ApiError when not authenticated", async () => {
            mockFetch.mockResolvedValue(mockErrorResponse(401, "Unauthorized"));

            const client = new ApiClient({ baseUrl: "http://localhost:3000" });
            const authApi = createAuthApi(client);

            await expect(
                authApi.updateMe({ bangumi_api_key: "test" })
            ).rejects.toThrow(ApiError);
        });
    });

    describe("updatePassword", () => {
        /**
         * **Validates: Requirement 2.4**
         * WHEN an authenticated user changes their password, THE Auth_API SHALL
         * send the password update request and return success status
         */

        it("sends password update request with correct data", async () => {
            mockFetch.mockResolvedValue({
                ok: true,
                status: 204,
                headers: new Headers({ "content-length": "0" }),
            });

            const passwordRequest: UpdatePasswordRequest = {
                old_password: "oldpass123",
                new_password: "newpass456",
            };

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.jwt.token",
            });
            const authApi = createAuthApi(client);

            await authApi.updatePassword(passwordRequest);

            expect(mockFetch).toHaveBeenCalledTimes(1);

            const [url, init] = mockFetch.mock.calls[0] as [
                string,
                RequestInit
            ];
            expect(url).toBe("http://localhost:3000/api/auth/password");
            expect(init.method).toBe("PUT");
            expect(JSON.parse(init.body as string)).toEqual(passwordRequest);

            const headers = init.headers as Record<string, string>;
            expect(headers["Authorization"]).toBe("Bearer valid.jwt.token");
        });

        it("throws ApiError when old password is incorrect", async () => {
            mockFetch.mockResolvedValue(
                mockErrorResponse(400, "Invalid old password")
            );

            const client = new ApiClient({
                baseUrl: "http://localhost:3000",
                getToken: () => "valid.jwt.token",
            });
            const authApi = createAuthApi(client);

            await expect(
                authApi.updatePassword({
                    old_password: "wrong",
                    new_password: "newpass",
                })
            ).rejects.toThrow(ApiError);

            try {
                await authApi.updatePassword({
                    old_password: "wrong",
                    new_password: "newpass",
                });
            } catch (error) {
                expect(error).toBeInstanceOf(ApiError);
                expect((error as ApiError).isBadRequest()).toBe(true);
            }
        });

        it("throws ApiError when not authenticated", async () => {
            mockFetch.mockResolvedValue(mockErrorResponse(401, "Unauthorized"));

            const client = new ApiClient({ baseUrl: "http://localhost:3000" });
            const authApi = createAuthApi(client);

            await expect(
                authApi.updatePassword({
                    old_password: "old",
                    new_password: "new",
                })
            ).rejects.toThrow(ApiError);
        });
    });
});
