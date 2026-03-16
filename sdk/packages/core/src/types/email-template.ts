export interface EmailTemplateWithContent {
  metadata: EmailTemplateMetadata;
  content: EmailTemplateContent;
  isCustomized: boolean;
  updatedAt?: string;
}

export interface EmailTemplateMetadata {
  templateType: string;
  name: string;
  description: string;
  variables: TemplateVariable[];
}

export interface TemplateVariable {
  name: string;
  description: string;
  example: string;
}

export interface EmailTemplateContent {
  subject: string;
  htmlBody: string;
  textBody: string;
}

export interface UpdateEmailTemplateInput {
  subject: string;
  htmlBody: string;
  textBody: string;
}

export interface PreviewEmailTemplateInput {
  subject: string;
  htmlBody: string;
  textBody: string;
}

export interface SendTestEmailTemplateInput {
  toEmail: string;
  subject: string;
  htmlBody: string;
  textBody: string;
  variables?: Record<string, string>;
}

export interface RenderedEmailPreview {
  subject: string;
  htmlBody: string;
  textBody: string;
}

export interface SendTestEmailResponse {
  success: boolean;
  message: string;
  messageId?: string;
}
