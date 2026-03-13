import type { ActionFunctionArgs, MetaFunction } from "react-router";
import { Form, useActionData, useNavigation, Link } from "react-router";
import { useState } from "react";
import { AuthPageShell } from "~/components/AuthPageShell";
import { Button } from "~/components/ui/button";
import { Input } from "~/components/ui/input";
import { Label } from "~/components/ui/label";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";
import { buildMeta, resolveMetaLocale } from "~/i18n/meta";
import { useI18n } from "~/i18n";
import { resolveLocale } from "~/services/locale.server";
import { translate } from "~/i18n/translate";
import { passwordApi } from "~/services/api";

export const meta: MetaFunction = ({ matches }) => {
  return buildMeta(resolveMetaLocale(matches), "auth.forgotPassword.metaTitle");
};

export async function action({ request }: ActionFunctionArgs) {
  const locale = await resolveLocale(request);
  const formData = await request.formData();
  const email = formData.get("email") as string;

  if (!email) {
    return { error: translate(locale, "auth.forgotPassword.emailRequired") };
  }

  try {
    await passwordApi.forgotPassword(email);
    return { success: true };
  } catch {
    // Don't reveal whether email exists - always show success
    return { success: true };
  }
}

export default function ForgotPasswordPage() {
  const { t } = useI18n();
  const actionData = useActionData<typeof action>();
  const navigation = useNavigation();
  const [email, setEmail] = useState("");

  const isSubmitting = navigation.state === "submitting";

  if (actionData?.success) {
    return (
      <AuthPageShell>
        <Card className="auth-form-card w-full max-w-md animate-fade-in-up">
          <CardHeader className="text-center">
            <div className="logo-icon mx-auto mb-4">A9</div>
            <CardTitle className="text-2xl">{t("auth.forgotPassword.successTitle")}</CardTitle>
            <CardDescription className="auth-form-description">
              {t("auth.forgotPassword.successDescription", { email })}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <p className="text-center text-sm text-[var(--text-secondary)]">
              {t("auth.forgotPassword.successHint")}{" "}
              <Link to="/forgot-password" className="font-medium text-[var(--accent-blue)] hover:underline">
                {t("auth.forgotPassword.tryAgain")}
              </Link>
              .
            </p>
            <div className="text-center">
              <Link to="/login">
                <Button variant="outline">{t("common.buttons.backToLogin")}</Button>
              </Link>
            </div>
          </CardContent>
        </Card>
      </AuthPageShell>
    );
  }

  return (
    <AuthPageShell>
      <Card className="auth-form-card w-full max-w-md animate-fade-in-up">
        <CardHeader className="text-center">
          <div className="logo-icon mx-auto mb-4">A9</div>
          <CardTitle className="text-2xl">{t("auth.forgotPassword.title")}</CardTitle>
          <CardDescription className="auth-form-description">{t("auth.forgotPassword.description")}</CardDescription>
        </CardHeader>
        <CardContent>
          <Form method="post" className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="email">{t("common.labels.emailAddress")}</Label>
              <Input
                id="email"
                name="email"
                type="email"
                placeholder={t("common.placeholders.email")}
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
                autoFocus
              />
            </div>

            {actionData?.error && (
              <div className="rounded-xl border border-[var(--accent-red)]/25 bg-[var(--accent-red)]/12 p-3 text-sm text-[var(--accent-red)]">
                {actionData.error}
              </div>
            )}

            <Button type="submit" className="w-full" disabled={isSubmitting}>
              {isSubmitting ? t("common.buttons.sending") : t("common.buttons.sendResetLink")}
            </Button>

            <div className="text-center text-sm">
              <Link to="/login" className="font-medium text-[var(--accent-blue)] hover:underline">
                {t("common.buttons.backToLogin")}
              </Link>
            </div>
          </Form>
        </CardContent>
      </Card>
    </AuthPageShell>
  );
}
