export interface BrandingConfig {
  logoUrl?: string;
  primaryColor: string;
  secondaryColor: string;
  backgroundColor: string;
  textColor: string;
  customCss?: string;
  companyName?: string;
  faviconUrl?: string;
  allowRegistration: boolean;
  emailOtpEnabled: boolean;
}

export interface UpdateBrandingInput {
  config: BrandingConfig;
}
