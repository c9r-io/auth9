import type { MetaFunction, ActionFunctionArgs, LoaderFunctionArgs } from "react-router";
import { redirect, Form, Link, useActionData, useLoaderData, useNavigation } from "react-router";
import { useState } from "react";
import { getBrandMark } from "~/components/auth/AuthBrandPanel";
import { AuthPageShell } from "~/components/AuthPageShell";
import { Button } from "~/components/ui/button";
import { Input } from "~/components/ui/input";
import { Label } from "~/components/ui/label";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";
import { buildMeta, resolveMetaLocale } from "~/i18n/meta";
import { useI18n } from "~/i18n";
import { resolveLocale } from "~/services/locale.server";
import { translate } from "~/i18n/translate";
import { mapApiError } from "~/lib/error-messages";
import { userApi, publicBrandingApi, captchaApi, type BrandingConfig } from "~/services/api";
import { DEFAULT_PUBLIC_BRANDING } from "~/services/api/branding";
import { Captcha, type CaptchaConfig, DEFAULT_CAPTCHA_CONFIG } from "~/components/captcha";

export const meta: MetaFunction = ({ matches }) => {
  return buildMeta(resolveMetaLocale(matches), "auth.register.metaTitle");
};

export async function loader(args?: LoaderFunctionArgs) {
  void args?.request;
  const clientId = process.env.AUTH9_PORTAL_CLIENT_ID || "auth9-portal";
  let branding: BrandingConfig = DEFAULT_PUBLIC_BRANDING;

  try {
    const { data } = await publicBrandingApi.get(clientId);
    branding = { ...DEFAULT_PUBLIC_BRANDING, ...data };
    if (!branding.allow_registration) {
      return redirect("/login");
    }
    let captchaConfig: CaptchaConfig = DEFAULT_CAPTCHA_CONFIG;
    try { captchaConfig = await captchaApi.getConfig(); } catch { /* ignore */ }
    return { branding, captchaConfig };
  } catch {
    // If we can't fetch branding config, default to disallowing registration
    return redirect("/login");
  }
}

export async function action({ request }: ActionFunctionArgs) {
  const locale = await resolveLocale(request);
  const formData = await request.formData();
  const email = formData.get("email");
  const password = formData.get("password");
  const displayName = formData.get("display_name");

  if (!email || !password) {
    return Response.json(
      { error: translate(locale, "auth.register.validationRequired") },
      { status: 400 }
    );
  }

  try {
    const result = await userApi.create({
      email: String(email),
      display_name: displayName ? String(displayName) : undefined,
      password: String(password),
    });
    const warning = result?.password_warning;
    if (warning) {
      return redirect(`/login?password_warning=${encodeURIComponent(warning)}`);
    }
    return redirect("/login");
  } catch (error) {
    return Response.json(
      { error: mapApiError(error, locale) },
      { status: 400 }
    );
  }
}

export default function Register() {
  const { t } = useI18n();
  const loaderData = (useLoaderData<typeof loader>() ?? {}) as { branding?: BrandingConfig; captchaConfig?: CaptchaConfig };
  const branding = { ...DEFAULT_PUBLIC_BRANDING, ...(loaderData.branding ?? {}) };
  const captchaConfig = loaderData.captchaConfig ?? DEFAULT_CAPTCHA_CONFIG;
  const actionData = useActionData<{ error?: string }>();
  const navigation = useNavigation();
  const isSubmitting = navigation.state === "submitting";
  const [captchaToken, setCaptchaToken] = useState("");

  return (
    <AuthPageShell
      branding={branding}
      panelEyebrow={t("auth.shared.hostedEyebrow")}
      panelTitle={t("auth.register.panelTitle")}
      panelDescription={t("auth.register.panelDescription")}
    >
      <Card className="w-full max-w-md animate-fade-in-up">
        <CardHeader className="text-center">
          {branding.logo_url ? (
            <img
              src={branding.logo_url}
              alt={branding.company_name || "Auth9"}
              className="mx-auto mb-4 h-14 w-14 rounded-2xl border border-black/5 bg-white/90 object-contain p-2"
              referrerPolicy="no-referrer"
              crossOrigin="anonymous"
            />
          ) : (
            <div className="logo-icon mx-auto mb-4">{getBrandMark(branding.company_name || "Auth9")}</div>
          )}
          <CardTitle className="text-2xl">{t("auth.register.title")}</CardTitle>
          <CardDescription>{t("auth.register.description")}</CardDescription>
        </CardHeader>
        <CardContent>
          <Form method="post" className="space-y-4">
            {actionData?.error && (
              <div className="p-3 rounded-xl bg-[var(--accent-red)]/10 text-[var(--accent-red)] text-sm border border-[var(--accent-red)]/20">
                {actionData.error}
              </div>
            )}

            <div className="space-y-2">
              <Label htmlFor="email">{t("common.labels.email")}</Label>
              <Input
                id="email"
                name="email"
                type="email"
                placeholder={t("common.placeholders.email")}
                required
                autoComplete="email"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="display_name">{t("common.labels.displayName")}</Label>
              <Input id="display_name" name="display_name" placeholder={t("common.placeholders.displayName")} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="password">{t("common.labels.password")}</Label>
              <Input
                id="password"
                name="password"
                type="password"
                placeholder={t("common.placeholders.passwordMasked")}
                required
                autoComplete="new-password"
              />
            </div>
            <input type="hidden" name="captchaToken" value={captchaToken} />
            {captchaConfig.enabled && captchaConfig.mode === "always" && (
              <Captcha config={captchaConfig} onVerify={setCaptchaToken} />
            )}
            <Button type="submit" className="w-full" disabled={isSubmitting}>
              {isSubmitting ? t("common.buttons.creating") : t("auth.register.submit")}
            </Button>
          </Form>
          <div className="mt-6 text-center text-sm text-[var(--text-tertiary)]">
            {t("auth.register.footerPrefix")}{" "}
            <Link to="/login" className="text-[var(--accent-blue)] hover:underline font-medium">
              {t("auth.register.footerAction")}
            </Link>
          </div>
          </CardContent>
        </Card>
    </AuthPageShell>
  );
}
