import type { Auth9HttpClient } from "../http-client.js";
import type {
  EmailTemplateWithContent,
  UpdateEmailTemplateInput,
  PreviewEmailTemplateInput,
  RenderedEmailPreview,
  SendTestEmailTemplateInput,
  SendTestEmailResponse,
} from "../types/email-template.js";

export class EmailTemplatesClient {
  constructor(private http: Auth9HttpClient) {}

  async list(): Promise<EmailTemplateWithContent[]> {
    const result = await this.http.get<{ data: EmailTemplateWithContent[] }>(
      "/api/v1/system/email-templates",
    );
    return result.data;
  }

  async get(type: string): Promise<EmailTemplateWithContent> {
    const result = await this.http.get<{ data: EmailTemplateWithContent }>(
      `/api/v1/system/email-templates/${type}`,
    );
    return result.data;
  }

  async update(
    type: string,
    input: UpdateEmailTemplateInput,
  ): Promise<EmailTemplateWithContent> {
    const result = await this.http.put<{ data: EmailTemplateWithContent }>(
      `/api/v1/system/email-templates/${type}`,
      input,
    );
    return result.data;
  }

  async reset(type: string): Promise<EmailTemplateWithContent> {
    const result = await this.http.delete<{ data: EmailTemplateWithContent }>(
      `/api/v1/system/email-templates/${type}`,
    );
    return result.data;
  }

  async preview(
    type: string,
    input: PreviewEmailTemplateInput,
  ): Promise<RenderedEmailPreview> {
    const result = await this.http.post<{ data: RenderedEmailPreview }>(
      `/api/v1/system/email-templates/${type}/preview`,
      input,
    );
    return result.data;
  }

  async sendTest(
    type: string,
    input: SendTestEmailTemplateInput,
  ): Promise<SendTestEmailResponse> {
    return this.http.post<SendTestEmailResponse>(
      `/api/v1/system/email-templates/${type}/send-test`,
      input,
    );
  }
}
