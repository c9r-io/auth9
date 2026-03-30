//! Email OTP channel implementation

use super::channel::{OtpChannel, OtpChannelType};
use crate::domains::platform::service::email::EmailService;
use crate::error::Result;
use crate::models::email::{EmailAddress, EmailMessage};
use crate::models::email_template::EmailTemplateType;
use crate::repository::SystemSettingsRepository;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// Email OTP channel wrapping the existing EmailService
pub struct EmailOtpChannel<R: SystemSettingsRepository> {
    email_service: Arc<EmailService<R>>,
}

impl<R: SystemSettingsRepository> EmailOtpChannel<R> {
    pub fn new(email_service: Arc<EmailService<R>>) -> Self {
        Self { email_service }
    }
}

#[async_trait]
impl<R: SystemSettingsRepository + 'static> OtpChannel for EmailOtpChannel<R> {
    fn channel_type(&self) -> OtpChannelType {
        OtpChannelType::Email
    }

    async fn send_code(&self, destination: &str, code: &str, ttl_minutes: u32) -> Result<()> {
        let mut vars = HashMap::new();
        vars.insert("user_name".to_string(), destination.to_string());
        vars.insert("verification_code".to_string(), code.to_string());
        vars.insert("expires_in_minutes".to_string(), ttl_minutes.to_string());
        vars.insert("app_name".to_string(), "Auth9".to_string());
        vars.insert(
            "year".to_string(),
            chrono::Utc::now().format("%Y").to_string(),
        );

        let rendered = self
            .email_service
            .resolve_and_render(EmailTemplateType::EmailMfa, &vars)
            .await?;

        let message = EmailMessage::new(
            EmailAddress::new(destination),
            &rendered.subject,
            &rendered.html_body,
        )
        .with_text_body(&rendered.text_body);

        self.email_service.send(&message, None).await?;
        Ok(())
    }
}
