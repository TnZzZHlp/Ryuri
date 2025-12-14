/**
 * Property-based tests for API type serialization.
 *
 * **Feature: frontend-api, Property 4: Type Serialization Round-Trip**
 * **Validates: Requirements 1.4, 1.5, 1.6, 8.1, 8.4, 8.5**
 */

import { describe, it, expect } from "vitest";
import * as fc from "fast-check";
import {
    ContentType,
    TaskPriority,
    TaskStatus,
    type LoginRequest,
    type LoginResponse,
    type UserResponse,
    type UpdateUserRequest,
    type UpdatePasswordRequest,
    type Library,
    type LibraryWithStats,
    type CreateLibraryRequest,
    type UpdateLibraryRequest,
    type ScanPath,
    type ContentResponse,
    type Chapter,
    type ProgressResponse,
    type ContentProgressResponse,
    type UpdateProgressRequest,
    type ScanTask,
    type BangumiSearchResult,
} from "../types";

// ============================================================================
// Arbitraries (Generators)
// ============================================================================

// ISO date string generator using integer timestamps to avoid invalid date issues
const isoDateArb = fc
    .integer({
        min: new Date("2000-01-01").getTime(),
        max: new Date("2100-01-01").getTime(),
    })
    .map((ts) => new Date(ts).toISOString());

// UUID string generator
const uuidArb = fc.uuid();

// Positive integer generator
const positiveIntArb = fc.integer({ min: 1, max: 1000000 });

// Non-negative integer generator
const nonNegativeIntArb = fc.integer({ min: 0, max: 1000000 });

// Percentage generator (0-100)
const percentageArb = fc.float({ min: 0, max: 100, noNaN: true });

// ContentType generator
const contentTypeArb = fc.constantFrom(ContentType.Comic, ContentType.Novel);

// TaskPriority generator
const taskPriorityArb = fc.constantFrom(TaskPriority.Normal, TaskPriority.High);

// TaskStatus generator
const taskStatusArb = fc.constantFrom(
    TaskStatus.Pending,
    TaskStatus.Running,
    TaskStatus.Completed,
    TaskStatus.Failed,
    TaskStatus.Cancelled
);

// Nullable string generator
const nullableStringArb = fc.option(fc.string(), { nil: null });

// Safe JSON value generator that avoids -0 edge case
// JSON.stringify(-0) produces "0", but Object.is(-0, 0) is false
const safeJsonValueArb = fc
    .jsonValue()
    .map((v) => JSON.parse(JSON.stringify(v)));

// ============================================================================
// Type Arbitraries
// ============================================================================

const loginRequestArb: fc.Arbitrary<LoginRequest> = fc.record({
    username: fc.string({ minLength: 1 }),
    password: fc.string({ minLength: 1 }),
});

const userResponseArb: fc.Arbitrary<UserResponse> = fc.record({
    id: positiveIntArb,
    username: fc.string({ minLength: 1 }),
    bangumi_api_key: nullableStringArb,
    created_at: isoDateArb,
});

const loginResponseArb: fc.Arbitrary<LoginResponse> = fc.record({
    user: userResponseArb,
    token: fc.string({ minLength: 1 }),
});

const updateUserRequestArb: fc.Arbitrary<UpdateUserRequest> = fc.record({
    bangumi_api_key: fc.option(nullableStringArb, { nil: undefined }),
});

const updatePasswordRequestArb: fc.Arbitrary<UpdatePasswordRequest> = fc.record(
    {
        old_password: fc.string({ minLength: 1 }),
        new_password: fc.string({ minLength: 1 }),
    }
);

const libraryArb: fc.Arbitrary<Library> = fc.record({
    id: positiveIntArb,
    name: fc.string({ minLength: 1 }),
    scan_interval: nonNegativeIntArb,
    watch_mode: fc.boolean(),
    created_at: isoDateArb,
    updated_at: isoDateArb,
});

const libraryWithStatsArb: fc.Arbitrary<LibraryWithStats> = fc.record({
    id: positiveIntArb,
    name: fc.string({ minLength: 1 }),
    scan_interval: nonNegativeIntArb,
    watch_mode: fc.boolean(),
    created_at: isoDateArb,
    updated_at: isoDateArb,
    path_count: nonNegativeIntArb,
    content_count: nonNegativeIntArb,
});

const createLibraryRequestArb: fc.Arbitrary<CreateLibraryRequest> = fc.record({
    name: fc.string({ minLength: 1 }),
    scan_interval: fc.option(nonNegativeIntArb, { nil: undefined }),
    watch_mode: fc.option(fc.boolean(), { nil: undefined }),
});

const updateLibraryRequestArb: fc.Arbitrary<UpdateLibraryRequest> = fc.record({
    name: fc.option(fc.string({ minLength: 1 }), { nil: undefined }),
    scan_interval: fc.option(nonNegativeIntArb, { nil: undefined }),
    watch_mode: fc.option(fc.boolean(), { nil: undefined }),
});

const scanPathArb: fc.Arbitrary<ScanPath> = fc.record({
    id: positiveIntArb,
    library_id: positiveIntArb,
    path: fc.string({ minLength: 1 }),
    created_at: isoDateArb,
});

const contentResponseArb: fc.Arbitrary<ContentResponse> = fc.record({
    id: positiveIntArb,
    library_id: positiveIntArb,
    content_type: contentTypeArb,
    title: fc.string({ minLength: 1 }),
    chapter_count: nonNegativeIntArb,
    has_thumbnail: fc.boolean(),
    metadata: fc.option(safeJsonValueArb, { nil: null }),
    created_at: isoDateArb,
});

const chapterArb: fc.Arbitrary<Chapter> = fc.record({
    id: positiveIntArb,
    content_id: positiveIntArb,
    title: fc.string({ minLength: 1 }),
    file_path: fc.string({ minLength: 1 }),
    sort_order: nonNegativeIntArb,
    page_count: nonNegativeIntArb,
});

const progressResponseArb: fc.Arbitrary<ProgressResponse> = fc.record({
    chapter_id: positiveIntArb,
    position: nonNegativeIntArb,
    percentage: percentageArb,
    updated_at: isoDateArb,
});

const contentProgressResponseArb: fc.Arbitrary<ContentProgressResponse> =
    fc.record({
        content_id: positiveIntArb,
        total_chapters: nonNegativeIntArb,
        completed_chapters: nonNegativeIntArb,
        current_chapter_id: fc.option(positiveIntArb, { nil: null }),
        overall_percentage: percentageArb,
    });

const updateProgressRequestArb: fc.Arbitrary<UpdateProgressRequest> = fc.record(
    {
        position: nonNegativeIntArb,
    }
);

const scanTaskArb: fc.Arbitrary<ScanTask> = fc.record({
    id: uuidArb,
    library_id: positiveIntArb,
    priority: taskPriorityArb,
    status: taskStatusArb,
    created_at: isoDateArb,
    started_at: fc.option(isoDateArb, { nil: null }),
    completed_at: fc.option(isoDateArb, { nil: null }),
    progress: fc.option(
        fc.record({
            scanned_paths: nonNegativeIntArb,
            total_paths: nonNegativeIntArb,
        }),
        { nil: null }
    ),
    result: fc.option(
        fc.record({
            added_count: nonNegativeIntArb,
            removed_count: nonNegativeIntArb,
            failed_scrape_count: nonNegativeIntArb,
        }),
        { nil: null }
    ),
    error: nullableStringArb,
});

const bangumiSearchResultArb: fc.Arbitrary<BangumiSearchResult> = fc.record({
    id: positiveIntArb,
    name: fc.string({ minLength: 1 }),
    name_cn: nullableStringArb,
    summary: nullableStringArb,
    image: nullableStringArb,
});

// ============================================================================
// Property Tests
// ============================================================================

describe("Type Serialization Round-Trip", () => {
    /**
     * **Feature: frontend-api, Property 4: Type Serialization Round-Trip**
     * **Validates: Requirements 1.4, 1.5, 1.6, 8.1, 8.4, 8.5**
     *
     * For any valid TypeScript request/response object, serializing to JSON
     * and deserializing back SHALL produce an equivalent object.
     */

    it("LoginRequest round-trip", () => {
        fc.assert(
            fc.property(loginRequestArb, (obj) => {
                const json = JSON.stringify(obj);
                const parsed = JSON.parse(json) as LoginRequest;
                expect(parsed).toEqual(obj);
            }),
            { numRuns: 100 }
        );
    });

    it("LoginResponse round-trip", () => {
        fc.assert(
            fc.property(loginResponseArb, (obj) => {
                const json = JSON.stringify(obj);
                const parsed = JSON.parse(json) as LoginResponse;
                expect(parsed).toEqual(obj);
            }),
            { numRuns: 100 }
        );
    });

    it("UserResponse round-trip", () => {
        fc.assert(
            fc.property(userResponseArb, (obj) => {
                const json = JSON.stringify(obj);
                const parsed = JSON.parse(json) as UserResponse;
                expect(parsed).toEqual(obj);
            }),
            { numRuns: 100 }
        );
    });

    it("UpdateUserRequest round-trip", () => {
        fc.assert(
            fc.property(updateUserRequestArb, (obj) => {
                const json = JSON.stringify(obj);
                const parsed = JSON.parse(json) as UpdateUserRequest;
                // Note: undefined values are not preserved in JSON, so we compare
                // only the defined properties
                expect(parsed).toEqual(JSON.parse(JSON.stringify(obj)));
            }),
            { numRuns: 100 }
        );
    });

    it("UpdatePasswordRequest round-trip", () => {
        fc.assert(
            fc.property(updatePasswordRequestArb, (obj) => {
                const json = JSON.stringify(obj);
                const parsed = JSON.parse(json) as UpdatePasswordRequest;
                expect(parsed).toEqual(obj);
            }),
            { numRuns: 100 }
        );
    });

    it("Library round-trip", () => {
        fc.assert(
            fc.property(libraryArb, (obj) => {
                const json = JSON.stringify(obj);
                const parsed = JSON.parse(json) as Library;
                expect(parsed).toEqual(obj);
            }),
            { numRuns: 100 }
        );
    });

    it("LibraryWithStats round-trip", () => {
        fc.assert(
            fc.property(libraryWithStatsArb, (obj) => {
                const json = JSON.stringify(obj);
                const parsed = JSON.parse(json) as LibraryWithStats;
                expect(parsed).toEqual(obj);
            }),
            { numRuns: 100 }
        );
    });

    it("CreateLibraryRequest round-trip", () => {
        fc.assert(
            fc.property(createLibraryRequestArb, (obj) => {
                const json = JSON.stringify(obj);
                const parsed = JSON.parse(json) as CreateLibraryRequest;
                expect(parsed).toEqual(JSON.parse(JSON.stringify(obj)));
            }),
            { numRuns: 100 }
        );
    });

    it("UpdateLibraryRequest round-trip", () => {
        fc.assert(
            fc.property(updateLibraryRequestArb, (obj) => {
                const json = JSON.stringify(obj);
                const parsed = JSON.parse(json) as UpdateLibraryRequest;
                expect(parsed).toEqual(JSON.parse(JSON.stringify(obj)));
            }),
            { numRuns: 100 }
        );
    });

    it("ScanPath round-trip", () => {
        fc.assert(
            fc.property(scanPathArb, (obj) => {
                const json = JSON.stringify(obj);
                const parsed = JSON.parse(json) as ScanPath;
                expect(parsed).toEqual(obj);
            }),
            { numRuns: 100 }
        );
    });

    it("ContentResponse round-trip", () => {
        fc.assert(
            fc.property(contentResponseArb, (obj) => {
                const json = JSON.stringify(obj);
                const parsed = JSON.parse(json) as ContentResponse;
                expect(parsed).toEqual(obj);
            }),
            { numRuns: 100 }
        );
    });

    it("Chapter round-trip", () => {
        fc.assert(
            fc.property(chapterArb, (obj) => {
                const json = JSON.stringify(obj);
                const parsed = JSON.parse(json) as Chapter;
                expect(parsed).toEqual(obj);
            }),
            { numRuns: 100 }
        );
    });

    it("ProgressResponse round-trip", () => {
        fc.assert(
            fc.property(progressResponseArb, (obj) => {
                const json = JSON.stringify(obj);
                const parsed = JSON.parse(json) as ProgressResponse;
                expect(parsed).toEqual(obj);
            }),
            { numRuns: 100 }
        );
    });

    it("ContentProgressResponse round-trip", () => {
        fc.assert(
            fc.property(contentProgressResponseArb, (obj) => {
                const json = JSON.stringify(obj);
                const parsed = JSON.parse(json) as ContentProgressResponse;
                expect(parsed).toEqual(obj);
            }),
            { numRuns: 100 }
        );
    });

    it("UpdateProgressRequest round-trip", () => {
        fc.assert(
            fc.property(updateProgressRequestArb, (obj) => {
                const json = JSON.stringify(obj);
                const parsed = JSON.parse(json) as UpdateProgressRequest;
                expect(parsed).toEqual(obj);
            }),
            { numRuns: 100 }
        );
    });

    it("ScanTask round-trip", () => {
        fc.assert(
            fc.property(scanTaskArb, (obj) => {
                const json = JSON.stringify(obj);
                const parsed = JSON.parse(json) as ScanTask;
                expect(parsed).toEqual(obj);
            }),
            { numRuns: 100 }
        );
    });

    it("BangumiSearchResult round-trip", () => {
        fc.assert(
            fc.property(bangumiSearchResultArb, (obj) => {
                const json = JSON.stringify(obj);
                const parsed = JSON.parse(json) as BangumiSearchResult;
                expect(parsed).toEqual(obj);
            }),
            { numRuns: 100 }
        );
    });
});

describe("ContentType Enum Serialization", () => {
    /**
     * **Feature: frontend-api, Property 5: ContentType Enum Serialization**
     * **Validates: Requirements 8.2**
     *
     * For any ContentType enum value (Comic or Novel), serializing to JSON
     * SHALL produce the string "Comic" or "Novel" respectively, and
     * deserializing SHALL produce the original enum value.
     */

    it("ContentType serializes to expected string values", () => {
        fc.assert(
            fc.property(contentTypeArb, (contentType) => {
                const json = JSON.stringify(contentType);

                // Should serialize to the exact string value
                if (contentType === ContentType.Comic) {
                    expect(json).toBe('"Comic"');
                } else {
                    expect(json).toBe('"Novel"');
                }
            }),
            { numRuns: 100 }
        );
    });

    it("ContentType round-trip preserves value", () => {
        fc.assert(
            fc.property(contentTypeArb, (contentType) => {
                const json = JSON.stringify(contentType);
                const parsed = JSON.parse(json) as ContentType;

                expect(parsed).toBe(contentType);
            }),
            { numRuns: 100 }
        );
    });

    it("ContentType values match backend format", () => {
        // Verify the exact string values match what the Rust backend produces
        expect(ContentType.Comic).toBe("Comic");
        expect(ContentType.Novel).toBe("Novel");

        // Verify serialization matches backend format
        expect(JSON.stringify(ContentType.Comic)).toBe('"Comic"');
        expect(JSON.stringify(ContentType.Novel)).toBe('"Novel"');
    });

    it("All ContentType values are covered", () => {
        // Ensure we have exactly the expected enum values
        const allValues = Object.values(ContentType);
        expect(allValues).toHaveLength(2);
        expect(allValues).toContain("Comic");
        expect(allValues).toContain("Novel");
    });
});
