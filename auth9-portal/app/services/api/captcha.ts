import { API_BASE_URL } from "./client";
import type { CaptchaConfig } from "~/components/captcha";
import { DEFAULT_CAPTCHA_CONFIG } from "~/components/captcha";

export const captchaApi = {
  /**
   * Fetch public CAPTCHA configuration (does not expose secret_key).
   * Falls back to disabled config on error.
   */
  getConfig: async (): Promise<CaptchaConfig> => {
    try {
      const response = await fetch(
        `${API_BASE_URL}/api/v1/public/captcha-config`
      );
      if (!response.ok) {
        return DEFAULT_CAPTCHA_CONFIG;
      }
      const data = await response.json();
      return {
        enabled: data.enabled ?? false,
        provider: data.provider ?? "turnstile",
        site_key: data.site_key ?? "",
        mode: data.mode ?? "disabled",
      };
    } catch {
      return DEFAULT_CAPTCHA_CONFIG;
    }
  },
};
