//! Security alert repository tests

use super::*;
use mockall::predicate::*;

#[tokio::test]
async fn test_mock_security_alert_repository() {
    let mut mock = MockSecurityAlertRepository::new();

    mock.expect_count_unresolved().returning(|| Ok(5));

    let count = mock.count_unresolved().await.unwrap();
    assert_eq!(count, 5);
}

#[tokio::test]
async fn test_mock_create() {
    let mut mock = MockSecurityAlertRepository::new();

    mock.expect_create().returning(|input| {
        Ok(SecurityAlert {
            user_id: input.user_id,
            tenant_id: input.tenant_id,
            alert_type: input.alert_type.clone(),
            severity: input.severity.clone(),
            details: input.details.clone(),
            ..Default::default()
        })
    });

    let input = CreateSecurityAlertInput {
        user_id: Some(StringUuid::new_v4()),
        tenant_id: None,
        alert_type: SecurityAlertType::BruteForce,
        severity: AlertSeverity::High,
        details: Some(serde_json::json!({"ip": "192.168.1.1", "attempts": 10})),
    };

    let alert = mock.create(&input).await.unwrap();
    assert_eq!(alert.alert_type, SecurityAlertType::BruteForce);
    assert_eq!(alert.severity, AlertSeverity::High);
}

#[tokio::test]
async fn test_mock_list_unresolved() {
    let mut mock = MockSecurityAlertRepository::new();

    mock.expect_list_unresolved()
        .with(eq(0), eq(10))
        .returning(|_, _| {
            Ok(vec![
                SecurityAlert {
                    alert_type: SecurityAlertType::BruteForce,
                    severity: AlertSeverity::Critical,
                    ..Default::default()
                },
                SecurityAlert {
                    alert_type: SecurityAlertType::NewDevice,
                    severity: AlertSeverity::Medium,
                    ..Default::default()
                },
            ])
        });

    let alerts = mock.list_unresolved(0, 10).await.unwrap();
    assert_eq!(alerts.len(), 2);
}

#[tokio::test]
async fn test_mock_resolve() {
    let mut mock = MockSecurityAlertRepository::new();
    let id = StringUuid::new_v4();
    let resolved_by = StringUuid::new_v4();

    mock.expect_resolve()
        .with(eq(id), eq(resolved_by))
        .returning(|id, resolved_by| {
            Ok(SecurityAlert {
                id,
                resolved_by: Some(resolved_by),
                resolved_at: Some(chrono::Utc::now()),
                ..Default::default()
            })
        });

    let alert = mock.resolve(id, resolved_by).await.unwrap();
    assert!(alert.resolved_at.is_some());
    assert_eq!(alert.resolved_by, Some(resolved_by));
}

#[tokio::test]
async fn test_mock_list_by_severity() {
    let mut mock = MockSecurityAlertRepository::new();

    mock.expect_list_by_severity()
        .with(eq(AlertSeverity::Critical), eq(0), eq(10))
        .returning(|severity, _, _| {
            Ok(vec![SecurityAlert {
                severity,
                ..Default::default()
            }])
        });

    let alerts = mock
        .list_by_severity(AlertSeverity::Critical, 0, 10)
        .await
        .unwrap();
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].severity, AlertSeverity::Critical);
}
