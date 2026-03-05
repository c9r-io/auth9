//! Invitation repository tests

use super::*;
use mockall::predicate::*;

#[tokio::test]
async fn test_mock_find_by_id() {
    let mut mock = MockInvitationRepository::new();
    let id = StringUuid::new_v4();
    let id_clone = id;

    mock.expect_find_by_id().with(eq(id)).returning(move |_| {
        Ok(Some(Invitation {
            id: id_clone,
            email: "test@example.com".to_string(),
            ..Default::default()
        }))
    });

    let result = mock.find_by_id(id).await.unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().email, "test@example.com");
}

#[tokio::test]
async fn test_mock_find_by_id_not_found() {
    let mut mock = MockInvitationRepository::new();

    mock.expect_find_by_id().returning(|_| Ok(None));

    let result = mock.find_by_id(StringUuid::new_v4()).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_mock_list_by_tenant() {
    let mut mock = MockInvitationRepository::new();
    let tenant_id = StringUuid::new_v4();

    mock.expect_list_by_tenant()
        .with(eq(tenant_id), eq(None), eq(0), eq(10))
        .returning(|_, _, _, _| {
            Ok(vec![
                Invitation {
                    email: "user1@example.com".to_string(),
                    ..Default::default()
                },
                Invitation {
                    email: "user2@example.com".to_string(),
                    ..Default::default()
                },
            ])
        });

    let result = mock.list_by_tenant(tenant_id, None, 0, 10).await.unwrap();
    assert_eq!(result.len(), 2);
}

#[tokio::test]
async fn test_mock_update_status() {
    let mut mock = MockInvitationRepository::new();
    let id = StringUuid::new_v4();
    let id_clone = id;

    mock.expect_update_status()
        .with(eq(id), eq(InvitationStatus::Revoked))
        .returning(move |_, status| {
            Ok(Invitation {
                id: id_clone,
                status,
                ..Default::default()
            })
        });

    let result = mock
        .update_status(id, InvitationStatus::Revoked)
        .await
        .unwrap();
    assert_eq!(result.status, InvitationStatus::Revoked);
}

#[tokio::test]
async fn test_mock_expire_pending() {
    let mut mock = MockInvitationRepository::new();

    mock.expect_expire_pending().returning(|| Ok(5));

    let result = mock.expire_pending().await.unwrap();
    assert_eq!(result, 5);
}
