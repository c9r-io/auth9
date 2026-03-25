import { useEffect, useRef, useCallback } from "react";

export interface CaptchaConfig {
  enabled: boolean;
  provider: "turnstile" | "recaptcha_v3" | "hcaptcha";
  site_key: string;
  mode: "always" | "adaptive" | "disabled";
}

export const DEFAULT_CAPTCHA_CONFIG: CaptchaConfig = {
  enabled: false,
  provider: "turnstile",
  site_key: "",
  mode: "disabled",
};

interface CaptchaProps {
  config: CaptchaConfig;
  onVerify: (token: string) => void;
  onError?: () => void;
  theme?: "light" | "dark" | "auto";
  size?: "normal" | "compact" | "invisible";
}

// Script loader to avoid duplicate script injections
const loadedScripts = new Set<string>();

function loadScript(src: string): Promise<void> {
  if (loadedScripts.has(src)) return Promise.resolve();
  return new Promise((resolve, reject) => {
    const script = document.createElement("script");
    script.src = src;
    script.async = true;
    script.defer = true;
    script.onload = () => {
      loadedScripts.add(src);
      resolve();
    };
    script.onerror = reject;
    document.head.appendChild(script);
  });
}

/**
 * CAPTCHA component supporting Cloudflare Turnstile, reCAPTCHA v3, and hCaptcha.
 *
 * Dynamically loads the provider script and renders the widget.
 * In "invisible" mode, triggers verification programmatically.
 */
export function Captcha({
  config,
  onVerify,
  onError,
  theme = "auto",
  size = "normal",
}: CaptchaProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const widgetIdRef = useRef<string | null>(null);

  const handleVerify = useCallback(
    (token: string) => {
      onVerify(token);
    },
    [onVerify]
  );

  const handleError = useCallback(() => {
    onError?.();
  }, [onError]);

  useEffect(() => {
    if (!config.enabled || !config.site_key || config.mode === "disabled") {
      return;
    }

    let cancelled = false;

    async function init() {
      try {
        if (config.provider === "turnstile") {
          await loadScript(
            "https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit"
          );
          if (cancelled || !containerRef.current) return;

          // Wait for turnstile to be available
          const w = window as unknown as {
            turnstile?: {
              render: (
                el: HTMLElement,
                opts: Record<string, unknown>
              ) => string;
              remove: (id: string) => void;
            };
          };
          if (w.turnstile && containerRef.current) {
            widgetIdRef.current = w.turnstile.render(containerRef.current, {
              sitekey: config.site_key,
              callback: handleVerify,
              "error-callback": handleError,
              theme,
              size,
            });
          }
        } else if (config.provider === "recaptcha_v3") {
          await loadScript(
            `https://www.google.com/recaptcha/api.js?render=${config.site_key}`
          );
          if (cancelled) return;

          const w = window as unknown as {
            grecaptcha?: {
              ready: (fn: () => void) => void;
              execute: (
                siteKey: string,
                opts: { action: string }
              ) => Promise<string>;
            };
          };
          w.grecaptcha?.ready(() => {
            w.grecaptcha
              ?.execute(config.site_key, { action: "submit" })
              .then(handleVerify)
              .catch(handleError);
          });
        } else if (config.provider === "hcaptcha") {
          await loadScript("https://js.hcaptcha.com/1/api.js?render=explicit");
          if (cancelled || !containerRef.current) return;

          const w = window as unknown as {
            hcaptcha?: {
              render: (
                el: HTMLElement,
                opts: Record<string, unknown>
              ) => string;
              remove: (id: string) => void;
            };
          };
          if (w.hcaptcha && containerRef.current) {
            widgetIdRef.current = w.hcaptcha.render(containerRef.current, {
              sitekey: config.site_key,
              callback: handleVerify,
              "error-callback": handleError,
              theme,
              size,
            });
          }
        }
      } catch {
        handleError();
      }
    }

    init();

    return () => {
      cancelled = true;
      // Cleanup widget on unmount
      if (widgetIdRef.current) {
        try {
          const w = window as unknown as Record<
            string,
            { remove?: (id: string) => void } | undefined
          >;
          if (config.provider === "turnstile") {
            w.turnstile?.remove?.(widgetIdRef.current);
          } else if (config.provider === "hcaptcha") {
            w.hcaptcha?.remove?.(widgetIdRef.current);
          }
        } catch {
          // ignore cleanup errors
        }
        widgetIdRef.current = null;
      }
    };
  }, [config, handleVerify, handleError, theme, size]);

  if (!config.enabled || config.mode === "disabled" || !config.site_key) {
    return null;
  }

  return <div ref={containerRef} className="flex justify-center my-3" />;
}
