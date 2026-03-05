//! Action repository tests

use super::*;

#[test]
fn test_action_repository_trait_is_mockable() {
    // This test ensures MockActionRepository can be created
    let _mock = MockActionRepository::new();
}
