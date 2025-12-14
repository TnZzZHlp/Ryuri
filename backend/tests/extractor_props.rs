//! Property tests for archive extractors.
//!
//! **Feature: comic-reader, Property 7: Chapter Sorting Consistency**
//! **Validates: Requirements 2.2, 2.3**
//!
//! For any set of chapter filenames, sorting them should produce a consistent order
//! where chapters are arranged by their natural sort order (handling numeric prefixes correctly).

use backend::extractors::natural_sort_key;
use proptest::prelude::*;

// ============================================================================
// Arbitrary Strategies for Filenames
// ============================================================================

/// Strategy to generate chapter-like filenames with numeric prefixes.
fn arb_chapter_filename() -> impl Strategy<Value = String> {
    prop_oneof![
        // Pattern: chapter01.cbz, chapter02.cbz, etc.
        (1u32..1000).prop_map(|n| format!("chapter{:02}.cbz", n)),
        // Pattern: vol01_ch01.zip, vol01_ch02.zip, etc.
        (1u32..100, 1u32..100).prop_map(|(v, c)| format!("vol{:02}_ch{:02}.zip", v, c)),
        // Pattern: 001.cbz, 002.cbz, etc.
        (1u32..1000).prop_map(|n| format!("{:03}.cbz", n)),
        // Pattern: page1.jpg, page2.jpg, etc.
        (1u32..1000).prop_map(|n| format!("page{}.jpg", n)),
        // Pattern with text prefix: manga_chapter_1.cbz
        (1u32..1000).prop_map(|n| format!("manga_chapter_{}.cbz", n)),
        // Pattern: Chapter 1.txt, Chapter 2.txt, etc.
        (1u32..1000).prop_map(|n| format!("Chapter {}.txt", n)),
    ]
}

/// Strategy to generate a vector of chapter filenames.
fn arb_chapter_filenames() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(arb_chapter_filename(), 1..50)
}

/// Strategy to generate mixed alphanumeric strings.
fn arb_mixed_string() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-z]{1,5}[0-9]{1,4}".prop_map(|s| s),
        "[0-9]{1,4}[a-z]{1,5}".prop_map(|s| s),
        "[a-z]{1,3}[0-9]{1,3}[a-z]{1,3}".prop_map(|s| s),
    ]
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 7: Chapter Sorting Consistency**
    /// **Validates: Requirements 2.2, 2.3**
    ///
    /// For any set of chapter filenames, sorting them twice should produce
    /// the same result (idempotence).
    #[test]
    fn sorting_is_idempotent(filenames in arb_chapter_filenames()) {
        let mut sorted1 = filenames.clone();
        sorted1.sort_by_key(|s| natural_sort_key(s));

        let mut sorted2 = sorted1.clone();
        sorted2.sort_by_key(|s| natural_sort_key(s));

        prop_assert_eq!(sorted1, sorted2, "Sorting should be idempotent");
    }

    /// **Feature: comic-reader, Property 7: Chapter Sorting Consistency**
    /// **Validates: Requirements 2.2, 2.3**
    ///
    /// For any set of chapter filenames, sorting should produce a total order
    /// (transitivity: if a <= b and b <= c, then a <= c).
    #[test]
    fn sorting_produces_total_order(filenames in arb_chapter_filenames()) {
        let mut sorted = filenames.clone();
        sorted.sort_by_key(|s| natural_sort_key(s));

        // Verify that the sorted list is actually sorted
        for i in 0..sorted.len().saturating_sub(1) {
            let key_a = natural_sort_key(&sorted[i]);
            let key_b = natural_sort_key(&sorted[i + 1]);
            prop_assert!(
                key_a <= key_b,
                "Element at index {} should be <= element at index {}: {:?} vs {:?}",
                i, i + 1, sorted[i], sorted[i + 1]
            );
        }
    }

    /// **Feature: comic-reader, Property 7: Chapter Sorting Consistency**
    /// **Validates: Requirements 2.2, 2.3**
    ///
    /// Natural sort should handle numeric prefixes correctly:
    /// "page2" should come before "page10".
    #[test]
    fn natural_sort_handles_numeric_prefixes(n1 in 1u32..100, n2 in 1u32..100) {
        let s1 = format!("page{}.jpg", n1);
        let s2 = format!("page{}.jpg", n2);

        let key1 = natural_sort_key(&s1);
        let key2 = natural_sort_key(&s2);

        // The comparison of keys should match the comparison of numbers
        prop_assert_eq!(
            key1.cmp(&key2),
            n1.cmp(&n2),
            "Natural sort key comparison should match numeric comparison for {} vs {}",
            s1, s2
        );
    }

    /// **Feature: comic-reader, Property 7: Chapter Sorting Consistency**
    /// **Validates: Requirements 2.2, 2.3**
    ///
    /// Natural sort should be case-insensitive for text portions.
    #[test]
    fn natural_sort_is_case_insensitive(s in "[a-zA-Z]{1,10}") {
        let lower = s.to_lowercase();
        let upper = s.to_uppercase();

        let key_lower = natural_sort_key(&lower);
        let key_upper = natural_sort_key(&upper);

        prop_assert_eq!(
            key_lower, key_upper,
            "Natural sort should be case-insensitive: {} vs {}",
            lower, upper
        );
    }

    /// **Feature: comic-reader, Property 7: Chapter Sorting Consistency**
    /// **Validates: Requirements 2.2, 2.3**
    ///
    /// For pure numeric strings, natural sort should match numeric comparison.
    #[test]
    fn natural_sort_numeric_strings(n1 in 0u64..10000, n2 in 0u64..10000) {
        let s1 = n1.to_string();
        let s2 = n2.to_string();

        let key1 = natural_sort_key(&s1);
        let key2 = natural_sort_key(&s2);

        prop_assert_eq!(
            key1.cmp(&key2),
            n1.cmp(&n2),
            "Natural sort should match numeric comparison for {} vs {}",
            s1, s2
        );
    }

    /// **Feature: comic-reader, Property 7: Chapter Sorting Consistency**
    /// **Validates: Requirements 2.2, 2.3**
    ///
    /// Sorting should preserve all elements (no elements lost or duplicated).
    #[test]
    fn sorting_preserves_elements(filenames in arb_chapter_filenames()) {
        let mut sorted = filenames.clone();
        sorted.sort_by_key(|s| natural_sort_key(s));

        // Same length
        prop_assert_eq!(
            filenames.len(),
            sorted.len(),
            "Sorting should preserve the number of elements"
        );

        // Same elements (as multiset)
        let mut original_sorted = filenames.clone();
        original_sorted.sort();
        let mut result_sorted = sorted.clone();
        result_sorted.sort();

        prop_assert_eq!(
            original_sorted,
            result_sorted,
            "Sorting should preserve all elements"
        );
    }

    /// **Feature: comic-reader, Property 7: Chapter Sorting Consistency**
    /// **Validates: Requirements 2.2, 2.3**
    ///
    /// Natural sort key should be deterministic - same input always produces same key.
    #[test]
    fn natural_sort_key_is_deterministic(s in arb_mixed_string()) {
        let key1 = natural_sort_key(&s);
        let key2 = natural_sort_key(&s);

        prop_assert_eq!(key1, key2, "Natural sort key should be deterministic for {}", s);
    }
}

// ============================================================================
// Additional Tests for Edge Cases
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Feature: comic-reader, Property 7: Chapter Sorting Consistency**
    /// **Validates: Requirements 2.2, 2.3**
    ///
    /// Volume-chapter patterns should sort correctly (vol1_ch1 < vol1_ch2 < vol2_ch1).
    #[test]
    fn volume_chapter_sorting(
        v1 in 1u32..50,
        c1 in 1u32..100,
        v2 in 1u32..50,
        c2 in 1u32..100
    ) {
        let s1 = format!("vol{:02}_ch{:02}.zip", v1, c1);
        let s2 = format!("vol{:02}_ch{:02}.zip", v2, c2);

        let key1 = natural_sort_key(&s1);
        let key2 = natural_sort_key(&s2);

        // Compare by volume first, then by chapter
        let expected = (v1, c1).cmp(&(v2, c2));

        prop_assert_eq!(
            key1.cmp(&key2),
            expected,
            "Volume-chapter sorting should be correct: {} vs {}",
            s1, s2
        );
    }

    /// **Feature: comic-reader, Property 7: Chapter Sorting Consistency**
    /// **Validates: Requirements 2.2, 2.3**
    ///
    /// Zero-padded numbers should sort the same as non-padded numbers.
    #[test]
    fn zero_padding_consistency(n in 1u32..1000) {
        let padded = format!("{:03}", n);
        let unpadded = format!("{}", n);

        let key_padded = natural_sort_key(&padded);
        let key_unpadded = natural_sort_key(&unpadded);

        prop_assert_eq!(
            key_padded, key_unpadded,
            "Zero-padded and unpadded numbers should have same sort key: {} vs {}",
            padded, unpadded
        );
    }
}
