import type { LoaderFunctionArgs, ActionFunctionArgs } from "react-router";
import { Form, Link, useActionData, useLoaderData, useNavigation } from "react-router";
import { useState, useEffect } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";
import { Button } from "~/components/ui/button";
import { Badge } from "~/components/ui/badge";
import { useI18n } from "~/i18n";
import { resolveLocale } from "~/services/locale.server";
import { translate } from "~/i18n/translate";
import { mapApiError } from "~/lib/error-messages";
import { adaptiveMfaApi, mfaApi, hostedLoginApi, type MfaStatusResponse, type TrustedDevice } from "~/services/api";
import { requireIdentityAuthWithUpdate } from "~/services/session.server";
import { LockClosedIcon, ArrowRightIcon } from "@radix-ui/react-icons";
import { TotpSetupInline } from "~/components/account/totp-setup-inline";
import { RecoveryCodesDialog } from "~/components/account/recovery-codes-dialog";
import { RemoveTotpDialog } from "~/components/account/remove-totp-dialog";

export async function loader({ request }: LoaderFunctionArgs) {
  try {
    const { session, headers } = await requireIdentityAuthWithUpdate(request);
    const accessToken = session.identityAccessToken || "";
    const status = await mfaApi.status(accessToken);

    let trustedDevices: TrustedDevice[] = [];
    try {
      const devicesResponse = await adaptiveMfaApi.listTrustedDevices(accessToken);
      trustedDevices = devicesResponse.data;
    } catch {
      // Trusted devices may not be available; ignore
    }

    const data = { status, accessToken, trustedDevices, error: null as string | null };
    if (headers) {
      return Response.json(data, { headers });
    }
    return data;
  } catch {
    const locale = await resolveLocale(request);
    return {
      status: {
        totp_enabled: false,
        webauthn_enabled: false,
        recovery_codes_remaining: 0,
        email_otp_enabled: false,
      } as MfaStatusResponse,
      accessToken: "",
      trustedDevices: [] as TrustedDevice[],
      error: translate(locale, "accountMfa.loadError"),
    };
  }
}

export async function action({ request }: ActionFunctionArgs) {
  const { session, headers } = await requireIdentityAuthWithUpdate(request);
  const accessToken = session.identityAccessToken || "";
  const locale = await resolveLocale(request);
  const formData = await request.formData();
  const intent = formData.get("intent");

  try {
    if (intent === "remove_totp") {
      await mfaApi.totpRemove(accessToken);
      const data = {
        success: true as const,
        intent: "remove_totp" as const,
        message: translate(locale, "accountMfa.totp.removeSuccess"),
        error: undefined as string | undefined,
        codes: undefined as string[] | undefined,
      };
      if (headers) return Response.json(data, { headers });
      return data;
    }

    if (intent === "verify_totp") {
      const setupToken = String(formData.get("setup_token") || "");
      const code = String(formData.get("code") || "").trim();
      await hostedLoginApi.totpEnrollVerify(setupToken, code, accessToken);
      const data = {
        success: true as const,
        intent: "verify_totp" as const,
        message: translate(locale, "accountMfa.totp.setupSuccess"),
        error: undefined as string | undefined,
        codes: undefined as string[] | undefined,
      };
      if (headers) return Response.json(data, { headers });
      return data;
    }

    if (intent === "enable_email_otp") {
      await mfaApi.emailOtpEnable(accessToken);
      const data = {
        success: true as const,
        intent: "enable_email_otp" as const,
        message: translate(locale, "accountMfa.emailOtp.enableSuccess"),
        error: undefined as string | undefined,
        codes: undefined as string[] | undefined,
      };
      if (headers) return Response.json(data, { headers });
      return data;
    }

    if (intent === "disable_email_otp") {
      await mfaApi.emailOtpDisable(accessToken);
      const data = {
        success: true as const,
        intent: "disable_email_otp" as const,
        message: translate(locale, "accountMfa.emailOtp.disableSuccess"),
        error: undefined as string | undefined,
        codes: undefined as string[] | undefined,
      };
      if (headers) return Response.json(data, { headers });
      return data;
    }

    if (intent === "revoke_trusted_device") {
      const deviceId = String(formData.get("device_id") || "");
      await adaptiveMfaApi.revokeTrustedDevice(deviceId, accessToken);
      const data = {
        success: true as const,
        intent: "revoke_trusted_device" as const,
        message: translate(locale, "trustedDevices.revokeSuccess"),
        error: undefined as string | undefined,
        codes: undefined as string[] | undefined,
      };
      if (headers) return Response.json(data, { headers });
      return data;
    }

    if (intent === "revoke_all_trusted_devices") {
      await adaptiveMfaApi.revokeAllTrustedDevices(accessToken);
      const data = {
        success: true as const,
        intent: "revoke_all_trusted_devices" as const,
        message: translate(locale, "trustedDevices.revokeSuccess"),
        error: undefined as string | undefined,
        codes: undefined as string[] | undefined,
      };
      if (headers) return Response.json(data, { headers });
      return data;
    }

    if (intent === "generate_recovery_codes") {
      const codes = await mfaApi.recoveryCodesGenerate(accessToken);
      const data = {
        success: true as const,
        intent: "generate_recovery_codes" as const,
        message: translate(locale, "accountMfa.recovery.generated"),
        error: undefined as string | undefined,
        codes: codes as string[] | undefined,
      };
      if (headers) return Response.json(data, { headers });
      return data;
    }
  } catch (error) {
    const message = mapApiError(error, locale);
    return {
      success: undefined as true | undefined,
      intent: undefined as string | undefined,
      message: undefined as string | undefined,
      error: message,
      codes: undefined as string[] | undefined,
    };
  }

  return {
    success: undefined as true | undefined,
    intent: undefined as string | undefined,
    message: undefined as string | undefined,
    error: translate(locale, "accountMfa.invalidAction"),
    codes: undefined as string[] | undefined,
  };
}

export default function AccountMfaPage() {
  const { t } = useI18n();
  const { status, accessToken, trustedDevices, error: loadError } = useLoaderData<typeof loader>();
  const actionData = useActionData<typeof action>();
  const navigation = useNavigation();
  const isSubmitting = navigation.state === "submitting";

  const [enrolling, setEnrolling] = useState(false);
  const [showRemoveDialog, setShowRemoveDialog] = useState(false);
  const [recoveryCodes, setRecoveryCodes] = useState<string[] | null>(null);

  // Pick up generated recovery codes from action response
  useEffect(() => {
    if (actionData?.success && actionData.intent === "generate_recovery_codes" && actionData.codes) {
      setRecoveryCodes(actionData.codes);
    }
  }, [actionData]);

  // Close inline TOTP setup on successful verification
  useEffect(() => {
    if (actionData?.success && actionData.intent === "verify_totp") {
      setEnrolling(false);
    }
  }, [actionData]);

  // Close remove dialog on successful removal
  useEffect(() => {
    if (actionData?.success && actionData.intent === "remove_totp") {
      setShowRemoveDialog(false);
    }
  }, [actionData]);

  return (
    <div className="space-y-6">
      {/* Error/Success Messages */}
      {loadError && (
        <div className="text-sm text-[var(--accent-red)] bg-red-50 p-3 rounded-md">
          {loadError}
        </div>
      )}

      {actionData?.error && (
        <div className="text-sm text-[var(--accent-red)] bg-red-50 p-3 rounded-md">
          {actionData.error}
        </div>
      )}

      {actionData?.success && actionData.intent !== "generate_recovery_codes" && (
        <div className="text-sm text-[var(--accent-green)] bg-[var(--accent-green)]/10 p-3 rounded-md">
          {actionData.message}
        </div>
      )}

      {/* TOTP Card */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>{t("accountMfa.totp.title")}</CardTitle>
              <CardDescription>{t("accountMfa.totp.description")}</CardDescription>
            </div>
            <Badge variant={status.totp_enabled ? "success" : "secondary"}>
              {status.totp_enabled ? t("accountMfa.totp.enabled") : t("accountMfa.totp.notEnabled")}
            </Badge>
          </div>
        </CardHeader>
        <CardContent>
          {status.totp_enabled ? (
            <Button
              variant="outline"
              className="text-[var(--accent-red)] hover:text-[var(--accent-red)] hover:bg-[var(--accent-red)]/10"
              onClick={() => setShowRemoveDialog(true)}
              disabled={isSubmitting}
            >
              {t("accountMfa.totp.remove")}
            </Button>
          ) : enrolling ? (
            <TotpSetupInline
              accessToken={accessToken}
              onCancel={() => setEnrolling(false)}
              error={actionData?.error && actionData.intent === undefined ? actionData.error : undefined}
            />
          ) : (
            <Button onClick={() => setEnrolling(true)}>
              {t("accountMfa.totp.setup")}
            </Button>
          )}
        </CardContent>
      </Card>

      {/* Email OTP Card */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>{t("accountMfa.emailOtp.title")}</CardTitle>
              <CardDescription>{t("accountMfa.emailOtp.description")}</CardDescription>
            </div>
            <Badge variant={status.email_otp_enabled ? "success" : "secondary"}>
              {status.email_otp_enabled
                ? t("accountMfa.emailOtp.enabled")
                : t("accountMfa.emailOtp.notEnabled")}
            </Badge>
          </div>
        </CardHeader>
        <CardContent>
          {status.email_otp_enabled ? (
            <Form method="post">
              <input type="hidden" name="intent" value="disable_email_otp" />
              <Button
                type="submit"
                variant="outline"
                className="text-[var(--accent-red)] hover:text-[var(--accent-red)] hover:bg-[var(--accent-red)]/10"
                disabled={isSubmitting}
              >
                {t("accountMfa.emailOtp.disable")}
              </Button>
            </Form>
          ) : (
            <Form method="post">
              <input type="hidden" name="intent" value="enable_email_otp" />
              <Button type="submit" disabled={isSubmitting}>
                {t("accountMfa.emailOtp.enable")}
              </Button>
            </Form>
          )}
        </CardContent>
      </Card>

      {/* Recovery Codes Card */}
      <Card>
        <CardHeader>
          <CardTitle>{t("accountMfa.recovery.title")}</CardTitle>
          <CardDescription>{t("accountMfa.recovery.description")}</CardDescription>
        </CardHeader>
        <CardContent>
          {status.totp_enabled ? (
            <div className="space-y-4">
              <div className="flex items-center gap-3">
                <span className="text-sm text-[var(--text-primary)]">
                  {t("accountMfa.recovery.remaining", { count: status.recovery_codes_remaining })}
                </span>
              </div>

              {status.recovery_codes_remaining < 3 && status.recovery_codes_remaining >= 0 && (
                <div className="text-sm text-[var(--accent-orange)] bg-[var(--accent-orange-light)] p-3 rounded-md">
                  {t("accountMfa.recovery.warning")}
                </div>
              )}

              <Form method="post">
                <input type="hidden" name="intent" value="generate_recovery_codes" />
                <Button type="submit" variant="outline" disabled={isSubmitting}>
                  {t("accountMfa.recovery.generate")}
                </Button>
              </Form>
            </div>
          ) : (
            <p className="text-sm text-[var(--text-tertiary)]">
              {t("accountMfa.recovery.notAvailable")}
            </p>
          )}
        </CardContent>
      </Card>

      {/* Trusted Devices Card */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>{t("trustedDevices.title")}</CardTitle>
              <CardDescription>{t("trustedDevices.description")}</CardDescription>
            </div>
            {trustedDevices.length > 0 && (
              <Form method="post">
                <input type="hidden" name="intent" value="revoke_all_trusted_devices" />
                <Button
                  type="submit"
                  variant="outline"
                  size="sm"
                  className="text-[var(--accent-red)] hover:text-[var(--accent-red)] hover:bg-[var(--accent-red)]/10"
                  disabled={isSubmitting}
                >
                  {t("trustedDevices.revokeAll")}
                </Button>
              </Form>
            )}
          </div>
        </CardHeader>
        <CardContent>
          {trustedDevices.length === 0 ? (
            <p className="text-sm text-[var(--text-tertiary)]">{t("trustedDevices.noDevices")}</p>
          ) : (
            <div className="space-y-3">
              {trustedDevices.filter((d) => !d.revoked).map((device) => (
                <div key={device.id} className="flex items-center justify-between rounded-lg border border-[var(--glass-border-subtle)] p-3">
                  <div className="space-y-1">
                    <p className="text-sm font-medium text-[var(--text-primary)]">
                      {device.device_name || device.device_fingerprint.slice(0, 12) + "..."}
                    </p>
                    <div className="flex gap-4 text-xs text-[var(--text-tertiary)]">
                      <span>{t("trustedDevices.trustedAt")}: {new Date(device.trusted_at).toLocaleDateString()}</span>
                      <span>{t("trustedDevices.expiresAt")}: {new Date(device.expires_at).toLocaleDateString()}</span>
                      <span>{t("trustedDevices.lastUsed")}: {new Date(device.last_used_at).toLocaleDateString()}</span>
                    </div>
                  </div>
                  <Form method="post">
                    <input type="hidden" name="intent" value="revoke_trusted_device" />
                    <input type="hidden" name="device_id" value={device.id} />
                    <Button
                      type="submit"
                      variant="outline"
                      size="sm"
                      className="text-[var(--accent-red)] hover:text-[var(--accent-red)] hover:bg-[var(--accent-red)]/10"
                      disabled={isSubmitting}
                    >
                      {t("trustedDevices.revoke")}
                    </Button>
                  </Form>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Passkeys Card */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>{t("accountMfa.passkeys.title")}</CardTitle>
              <CardDescription>{t("accountMfa.passkeys.description")}</CardDescription>
            </div>
            <Badge variant={status.webauthn_enabled ? "success" : "secondary"}>
              {status.webauthn_enabled ? t("accountMfa.passkeys.enabled") : t("accountMfa.passkeys.notEnabled")}
            </Badge>
          </div>
        </CardHeader>
        <CardContent>
          <Link
            to="/dashboard/account/passkeys"
            className="inline-flex items-center gap-1 text-sm font-medium text-[var(--accent-blue)] hover:underline"
          >
            <LockClosedIcon className="h-4 w-4" />
            {t("accountMfa.passkeys.manage")}
            <ArrowRightIcon className="h-3 w-3" />
          </Link>
        </CardContent>
      </Card>

      {/* Dialogs */}
      <RemoveTotpDialog open={showRemoveDialog} onOpenChange={setShowRemoveDialog} />

      {recoveryCodes && (
        <RecoveryCodesDialog
          codes={recoveryCodes}
          open={true}
          onClose={() => setRecoveryCodes(null)}
        />
      )}
    </div>
  );
}
