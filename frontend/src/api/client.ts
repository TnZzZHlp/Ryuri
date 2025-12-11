/**
 * API Client Module
 *
 * Core HTTP client that wraps fetch API, providing unified request/response handling,
 * authentication header injection, and error parsing.
 *
 * **Implements: Requirements 1.1, 1.2, 1.3, 1.4, 1.5, 1.6**
 */

import { ApiError } from "./types";

/**
 * Configuration options for the API client.
 */
export interface ApiClientConfig {
    /** Base URL for all API requests */
    baseUrl: string;
    /** Optional function to retrieve the current auth token */
    getToken?: () => string | null;
}

/**
 * Options for individual requests.
 */
export interface RequestOptions {
    /** Query parameters to append to the URL */
    params?: Record<string, string | number | boolean | undefined>;
    /** Additional headers to include */
    headers?: Record<string, string>;
    /** Whether this request requires authentication (default: true) */
    requiresAuth?: boolean;
}

/**
 * Constructs a full URL from base URL and path.
 * Ensures proper handling of trailing/leading slashes.
 */
export function buildUrl(baseUrl: string, path: string): string {
    // Remove trailing slash from baseUrl
    const normalizedBase = baseUrl.endsWith("/")
        ? baseUrl.slice(0, -1)
        : baseUrl;
    // Ensure path starts with /
    const normalizedPath = path.startsWith("/") ? path : `/${path}`;
    return `${normalizedBase}${normalizedPath}`;
}

/**
 * Appends query parameters to a URL.
 */
export function appendQueryParams(
    url: string,
    params?: Record<string, string | number | boolean | undefined>
): string {
    if (!params) return url;

    const searchParams = new URLSearchParams();
    for (const [key, value] of Object.entries(params)) {
        if (value !== undefined) {
            searchParams.append(key, String(value));
        }
    }

    const queryString = searchParams.toString();
    if (!queryString) return url;

    return url.includes("?")
        ? `${url}&${queryString}`
        : `${url}?${queryString}`;
}

/**
 * Builds the Authorization header value for a given token.
 */
export function buildAuthHeader(token: string): string {
    return `Bearer ${token}`;
}

/**
 * Parses an error response body and extracts the error message.
 * Backend returns errors in format: { "error": "message" }
 */
async function parseErrorResponse(response: Response): Promise<string> {
    try {
        const body = await response.text();
        if (!body) {
            return response.statusText || "Unknown error";
        }

        try {
            const json = JSON.parse(body) as { error?: string };
            return json.error || body;
        } catch {
            // If not valid JSON, return the raw text
            return body;
        }
    } catch {
        return response.statusText || "Unknown error";
    }
}

/**
 * API Client class that handles all HTTP communication with the backend.
 */
export class ApiClient {
    private readonly _baseUrl: string;
    private readonly getToken: () => string | null;

    constructor(config: ApiClientConfig) {
        this._baseUrl = config.baseUrl;
        this.getToken = config.getToken ?? (() => null);
    }

    /**
     * Gets the configured base URL.
     */
    get baseUrl(): string {
        return this._baseUrl;
    }

    /**
     * Performs a GET request.
     */
    async get<T>(path: string, options?: RequestOptions): Promise<T> {
        return this.request<T>("GET", path, undefined, options);
    }

    /**
     * Performs a POST request.
     */
    async post<T>(
        path: string,
        body?: unknown,
        options?: RequestOptions
    ): Promise<T> {
        return this.request<T>("POST", path, body, options);
    }

    /**
     * Performs a PUT request.
     */
    async put<T>(
        path: string,
        body?: unknown,
        options?: RequestOptions
    ): Promise<T> {
        return this.request<T>("PUT", path, body, options);
    }

    /**
     * Performs a DELETE request.
     */
    async delete<T>(path: string, options?: RequestOptions): Promise<T> {
        return this.request<T>("DELETE", path, undefined, options);
    }

    /**
     * Core request method that handles all HTTP operations.
     */
    private async request<T>(
        method: string,
        path: string,
        body?: unknown,
        options?: RequestOptions
    ): Promise<T> {
        // Build the full URL
        let url = buildUrl(this._baseUrl, path);
        url = appendQueryParams(url, options?.params);

        // Build headers
        const headers: Record<string, string> = {
            "Content-Type": "application/json",
            ...options?.headers,
        };

        // Add auth header if required (default: true)
        const requiresAuth = options?.requiresAuth ?? true;
        if (requiresAuth) {
            const token = this.getToken();
            if (token) {
                headers["Authorization"] = buildAuthHeader(token);
            }
        }

        // Build request init
        const init: RequestInit = {
            method,
            headers,
        };

        // Add body for non-GET requests
        if (body !== undefined) {
            init.body = JSON.stringify(body);
        }

        // Execute the request
        let response: Response;
        try {
            response = await fetch(url, init);
        } catch (error) {
            // Network error
            throw new ApiError(
                0,
                error instanceof Error ? error.message : "Network error"
            );
        }

        // Handle error responses
        if (!response.ok) {
            const message = await parseErrorResponse(response);
            throw new ApiError(response.status, message);
        }

        // Handle empty responses (204 No Content)
        if (
            response.status === 204 ||
            response.headers.get("content-length") === "0"
        ) {
            return undefined as T;
        }

        // Parse JSON response
        try {
            const data = (await response.json()) as T;
            return data;
        } catch {
            throw new ApiError(
                response.status,
                "Failed to parse response JSON"
            );
        }
    }
}

/**
 * Default API client instance.
 * Can be configured by setting the token getter.
 */
let defaultClient: ApiClient | null = null;

/**
 * Creates or returns the default API client.
 */
export function getDefaultClient(config?: ApiClientConfig): ApiClient {
    if (!defaultClient && config) {
        defaultClient = new ApiClient(config);
    }
    if (!defaultClient) {
        throw new Error("API client not initialized. Call with config first.");
    }
    return defaultClient;
}

/**
 * Resets the default client (useful for testing).
 */
export function resetDefaultClient(): void {
    defaultClient = null;
}
