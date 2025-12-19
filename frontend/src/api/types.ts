/**
 * Frontend API Type Definitions
 *
 * This module defines all TypeScript types for API requests and responses,
 * matching the backend Rust schemas.
 */

// ============================================================================
// Error Types
// ============================================================================

/**
 * API error class for handling HTTP errors from the backend.
 */
export class ApiError extends Error {
    readonly status: number;
    readonly details?: unknown;

    constructor(status: number, message: string, details?: unknown) {
        super(message);
        this.name = "ApiError";
        this.status = status;
        this.details = details;
    }

    isUnauthorized(): boolean {
        return this.status === 401;
    }

    isNotFound(): boolean {
        return this.status === 404;
    }

    isBadRequest(): boolean {
        return this.status === 400;
    }

    isServerError(): boolean {
        return this.status >= 500;
    }
}

// ============================================================================
// Enums (as const objects for erasableSyntaxOnly compatibility)
// ============================================================================

/**
 * Content type enumeration.
 * Distinguishes between comic (image-based) and novel (text-based) content.
 */
export const ContentType = {
    Comic: "Comic",
    Novel: "Novel",
} as const;

export type ContentType = (typeof ContentType)[keyof typeof ContentType];

/**
 * Task priority for scan operations.
 */
export const TaskPriority = {
    Normal: "Normal",
    High: "High",
} as const;

export type TaskPriority = (typeof TaskPriority)[keyof typeof TaskPriority];

/**
 * Task status for scan operations.
 */
export const TaskStatus = {
    Pending: "Pending",
    Running: "Running",
    Completed: "Completed",
    Failed: "Failed",
    Cancelled: "Cancelled",
} as const;

export type TaskStatus = (typeof TaskStatus)[keyof typeof TaskStatus];

// ============================================================================
// Authentication Types
// ============================================================================

/**
 * Request for user login.
 */
export interface LoginRequest {
    username: string;
    password: string;
}

/**
 * Response for successful login.
 */
export interface LoginResponse {
    user: UserResponse;
    token: string;
}

/**
 * User data for API responses (without sensitive fields).
 */
export interface UserResponse {
    id: number;
    username: string;
    bangumi_api_key: string | null;
    created_at: string;
}

/**
 * Request to update user information.
 */
export interface UpdateUserRequest {
    bangumi_api_key?: string | null;
}

/**
 * Request to update password.
 */
export interface UpdatePasswordRequest {
    old_password: string;
    new_password: string;
}

/**
 * Request to create a new API key.
 */
export interface CreateApiKeyRequest {
    name: string;
}

/**
 * Response for API key generation and listing.
 */
export interface ApiKeyResponse {
    id: number;
    name: string;
    api_key: string;
    created_at: string;
}

// ============================================================================
// Library Types
// ============================================================================

/**
 * A content library that can contain multiple scan paths.
 */
export interface Library {
    id: number;
    name: string;
    scan_interval: number;
    watch_mode: boolean;
    created_at: string;
    updated_at: string;
}

/**
 * Library with computed statistics.
 */
export interface LibraryWithStats extends Library {
    path_count: number;
    content_count: number;
}

/**
 * Request to create a new library.
 */
export interface CreateLibraryRequest {
    name: string;
    scan_interval?: number;
    watch_mode?: boolean;
}

/**
 * Request to update an existing library.
 */
export interface UpdateLibraryRequest {
    name?: string;
    scan_interval?: number;
    watch_mode?: boolean;
}

/**
 * A scan path associated with a library.
 */
export interface ScanPath {
    id: number;
    library_id: number;
    path: string;
    created_at: string;
}

// ============================================================================
// Content Types
// ============================================================================

/**
 * Response structure for content list API.
 */
export interface ContentResponse {
    id: number;
    library_id: number;
    content_type: ContentType;
    title: string;
    chapter_count: number;
    has_thumbnail: boolean;
    metadata: unknown | null;
    created_at: string;
}

/**
 * A chapter within a content item.
 */
export interface Chapter {
    id: number;
    content_id: number;
    title: string;
    file_path: string;
    sort_order: number;
    page_count: number;
    size: number;
}

/**
 * Response for chapter text content.
 */
export interface ChapterTextResponse {
    text: string;
}

/**
 * Request to update content information.
 */
export interface UpdateContentRequest {
    title?: string;
    metadata?: unknown | null;
}

// ============================================================================
// Progress Types
// ============================================================================

/**
 * Response for chapter reading progress API.
 */
export interface ProgressResponse {
    chapter_id: number;
    position: number;
    percentage: number;
    updated_at: string;
}

/**
 * Response for overall content progress (aggregated from chapter progress).
 */
export interface ContentProgressResponse {
    content_id: number;
    total_chapters: number;
    completed_chapters: number;
    current_chapter_id: number | null;
    overall_percentage: number;
}

/**
 * Request to update reading progress for a chapter.
 */
export interface UpdateProgressRequest {
    position: number;
}

// ============================================================================
// Scan Queue Types
// ============================================================================

/**
 * Progress information for a running scan task.
 */
export interface TaskProgress {
    scanned_paths: number;
    total_paths: number;
}

/**
 * Result information for a completed scan task.
 */
export interface TaskResult {
    added_count: number;
    removed_count: number;
    failed_scrape_count: number;
}

/**
 * A scan task representing a queued or executed scan operation.
 */
export interface ScanTask {
    id: string;
    library_id: number;
    priority: TaskPriority;
    status: TaskStatus;
    created_at: string;
    started_at: string | null;
    completed_at: string | null;
    progress: TaskProgress | null;
    result: TaskResult | null;
    error: string | null;
}

/**
 * Response for submitting a scan task.
 */
export interface SubmitScanResponse {
    task_id: string;
    task: ScanTask;
}

// ============================================================================
// Bangumi Types
// ============================================================================

/**
 * Search result from Bangumi API.
 */
export interface BangumiSearchResult {
    id: number;
    name: string;
    name_cn: string | null;
    summary: string | null;
    image: string | null;
}
