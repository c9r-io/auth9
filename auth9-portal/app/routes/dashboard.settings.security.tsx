import type { ActionFunctionArgs, LoaderFunctionArgs, MetaFunction } from "react-router";
import { Form, useActionData, useFetcher, useLoaderData, useNavigation } from "react-router";
import { useEffect, useState } from "react";
import { SettingsSectionHeading } from "~/components/settings/settings-card-header";
import { Button } from "~/components/ui/button";
import { Card, CardContent, CardHeader } from "~/components/ui/card";
import { Input } from "~/components/ui/input";
import { Label } from "~/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/select";
import { Switch } from "~/components/ui/switch";
import { useI18n } from "~/i18n";
import { buildMeta, resolveMetaLocale } from "~/i18n/meta";
import { translate } from "~/i18n/translate";
import { mapApiError } from "~/lib/error-messages";
import {
  passwordApi,
  systemApi,
  tenantApi,
  type MaliciousIpBlacklistEntry,
  type PasswordPolicy,
  type Tenant,
} from "~/services/api";
import { getAccessToken } from "~/services/session.server";
import { resolveLocale } from "~/services/locale.server";

export const meta: MetaFunction = ({ matches }) => buildMeta(resolveMetaLocale(matches), "settings.securitySettings.metaTitle");

export async function loader({ request }: LoaderFunctionArgs) {
  const locale = await resolveLocale(request);
  const accessToken = await getAccessToken(request);
  const url = new URL(request.url);
  const tenantId = url.searchParams.get("tenantId");
  const isDataRequest = url.pathname.endsWith(".data");

  let tenants: Tenant[] = [];
  let tenantsError: string | null = null;

  if (!isDataRequest) {
    try {
      const tenantsResponse = await tenantApi.list(1, 100, undefined, accessToken || undefined);
      tenants = tenantsResponse.data;
    } catch (error) {
      tenantsError = error instanceof Error ? error.message : translate(locale, "settings.securitySettings.loadTenantsFailed");
    }
  }

  let policy: PasswordPolicy | null = null;
  let policyError: string | null = null;
  let blacklist: MaliciousIpBlacklistEntry[] = [];
  let blacklistError: string | null = null;

  if (tenantId) {
    try {
      const policyResponse = await passwordApi.getPasswordPolicy(tenantId, accessToken || undefined);
      policy = policyResponse.data;
    } catch (error) {
      policyError = error instanceof Error ? error.message : translate(locale, "settings.securitySettings.loadPolicyFailed");
    }
  }

  try {
    const blacklistResponse = await systemApi.getMaliciousIpBlacklist(accessToken || undefined);
    blacklist = blacklistResponse.data;
  } catch (error) {
    blacklistError = error instanceof Error ? error.message : translate(locale, "settings.securitySettings.loadBlacklistFailed");
  }

  return {
    tenants,
    tenantsError,
    selectedTenantId: tenantId || "",
    policy,
    policyError,
    blacklist,
    blacklistError,
  };
}

export async function action({ request }: ActionFunctionArgs) {
  const locale = await resolveLocale(request);
  const accessToken = await getAccessToken(request);
  const formData = await request.formData();
  const intent = formData.get("intent");

  try {
    if (intent === "update_policy") {
      const tenantId = formData.get("tenantId") as string;
      const policy: Partial<PasswordPolicy> = {
        min_length: parseInt(formData.get("minLength") as string) || 8,
        require_uppercase: formData.get("requireUppercase") === "true",
        require_lowercase: formData.get("requireLowercase") === "true",
        require_numbers: formData.get("requireNumbers") === "true",
        require_symbols: formData.get("requireSymbols") === "true",
        max_age_days: parseInt(formData.get("maxAgeDays") as string) || 0,
        history_count: parseInt(formData.get("historyCount") as string) || 0,
        lockout_threshold: parseInt(formData.get("lockoutThreshold") as string) || 0,
        lockout_duration_mins: parseInt(formData.get("lockoutDurationMins") as string) || 15,
      };

      await passwordApi.updatePasswordPolicy(tenantId, policy, accessToken || undefined);
      return { success: true, message: translate(locale, "settings.securitySettings.updated") };
    }

    if (intent === "update_malicious_ip_blacklist") {
      const raw = (formData.get("maliciousIps") as string) || "";
      const entries = raw
        .split(/\r?\n/)
        .map((line) => line.trim())
        .filter(Boolean)
        .map((ip_address) => ({ ip_address }));

      await systemApi.updateMaliciousIpBlacklist(entries, accessToken || undefined);
      return { success: true, message: translate(locale, "settings.securitySettings.blacklistUpdated") };
    }
  } catch (error) {
    const message = mapApiError(error, locale);
    return { error: message };
  }

  return { error: translate(locale, "settings.securitySettings.invalidAction") };
}

export default function SecuritySettingsPage() {
  const { tenants, tenantsError, selectedTenantId, policy: loadedPolicy, policyError, blacklist, blacklistError } = useLoaderData<typeof loader>();
  const actionData = useActionData<typeof action>();
  const navigation = useNavigation();
  const policyFetcher = useFetcher<typeof loader>();
  const { t } = useI18n();

  const [selectedTenant, setSelectedTenant] = useState<string>(selectedTenantId);
  const [policy, setPolicy] = useState<PasswordPolicy | null>(loadedPolicy);
  const [blacklistText, setBlacklistText] = useState<string>(blacklist.map((entry) => entry.ip_address).join("\n"));

  const isSubmitting = navigation.state === "submitting";
  const loadingPolicy = policyFetcher.state === "loading";
  const loadPolicy = policyFetcher.load;

  useEffect(() => {
    if (selectedTenant) {
      loadPolicy(`/dashboard/settings/security?tenantId=${encodeURIComponent(selectedTenant)}`);
    } else {
      setPolicy(null);
    }
  }, [loadPolicy, selectedTenant]);

  useEffect(() => {
    if (policyFetcher.data) {
      setPolicy(policyFetcher.data.policy || null);
    }
  }, [policyFetcher.data]);

  useEffect(() => {
    setBlacklistText(blacklist.map((entry) => entry.ip_address).join("\n"));
  }, [blacklist]);

  const syncHiddenBooleanField = (fieldName: string, checked: boolean) => {
    const input = document.getElementById(fieldName) as HTMLInputElement | null;
    if (input) {
      input.value = checked ? "true" : "false";
    }
  };

  const actionBarClassName =
    "sticky bottom-0 z-10 -mx-6 -mb-6 flex flex-wrap items-center gap-3 border-t border-[var(--glass-border-subtle)] bg-[var(--bg-secondary)]/95 px-6 py-4 backdrop-blur supports-[backdrop-filter]:bg-[var(--bg-secondary)]/80 md:static md:mx-0 md:mb-0 md:border-t-0 md:bg-transparent md:px-0 md:py-0 md:backdrop-blur-none";

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader className="p-5 pb-5 sm:p-6 sm:pb-6">
          <SettingsSectionHeading
            title={t("settings.securitySettings.blacklistTitle")}
            description={t("settings.securitySettings.blacklistDescription")}
          />
        </CardHeader>
        <CardContent>
          <Form method="post" className="space-y-4">
            <input type="hidden" name="intent" value="update_malicious_ip_blacklist" />
            <div className="space-y-2">
              <Label htmlFor="maliciousIps">{t("settings.securitySettings.blacklistInput")}</Label>
              <textarea
                id="maliciousIps"
                name="maliciousIps"
                value={blacklistText}
                onChange={(event) => setBlacklistText(event.target.value)}
                className="min-h-40 w-full rounded-md border border-gray-300 px-3 py-2 text-sm"
                placeholder={t("settings.securitySettings.blacklistPlaceholder")}
              />
              <p className="text-xs text-[var(--text-secondary)]">{t("settings.securitySettings.blacklistHint")}</p>
            </div>

            {blacklistError && <div className="rounded-md bg-red-50 p-3 text-sm text-[var(--accent-red)]">{blacklistError}</div>}
            {actionData?.error && <div className="rounded-md bg-red-50 p-3 text-sm text-[var(--accent-red)]">{actionData.error}</div>}
            {actionData?.success && <div className="rounded-md bg-[var(--accent-green)]/10 p-3 text-sm text-[var(--accent-green)]">{actionData.message}</div>}

            <div className={actionBarClassName}>
              <Button type="submit" disabled={isSubmitting}>{isSubmitting ? t("settings.securitySettings.saving") : t("settings.securitySettings.saveBlacklist")}</Button>
            </div>
          </Form>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="p-5 pb-5 sm:p-6 sm:pb-6">
          <SettingsSectionHeading
            title={t("settings.securitySettings.title")}
            description={t("settings.securitySettings.description")}
          />
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="max-w-xs space-y-2">
              <Label id="tenantSelectLabel" htmlFor="tenantSelectTrigger">{t("settings.securitySettings.selectTenant")}</Label>
              <Select
                value={selectedTenant || "__none__"}
                onValueChange={(value) => setSelectedTenant(value === "__none__" ? "" : value)}
              >
                <SelectTrigger id="tenantSelectTrigger" aria-labelledby="tenantSelectLabel">
                  <SelectValue placeholder={t("settings.securitySettings.selectTenantPlaceholder")} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="__none__">{t("settings.securitySettings.selectTenantPlaceholder")}</SelectItem>
                  {tenants.map((tenant) => (
                    <SelectItem key={tenant.id} value={tenant.id}>
                      {tenant.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {tenantsError && <div className="rounded-md bg-red-50 p-3 text-sm text-[var(--accent-red)]">{tenantsError}</div>}
            {loadingPolicy && <p className="text-sm text-[var(--text-secondary)]">{t("settings.securitySettings.loadingPolicy")}</p>}
            {selectedTenant && policyError && !policy && <div className="rounded-md bg-red-50 p-3 text-sm text-[var(--accent-red)]">{policyError}</div>}

            {selectedTenant && policy && (
              <Form method="post" className="space-y-6">
                <input type="hidden" name="intent" value="update_policy" />
                <input type="hidden" name="tenantId" value={selectedTenant} />

                <div className="grid gap-6 md:grid-cols-2">
                  <div className="space-y-2"><Label htmlFor="minLength">{t("settings.securitySettings.minimumLength")}</Label><Input id="minLength" name="minLength" type="number" min={6} max={128} defaultValue={policy.min_length} /></div>
                  <div className="space-y-2"><Label htmlFor="maxAgeDays">{t("settings.securitySettings.passwordExpiryDays")}</Label><Input id="maxAgeDays" name="maxAgeDays" type="number" min={0} max={365} defaultValue={policy.max_age_days} /><p className="text-xs text-[var(--text-secondary)]">{t("settings.securitySettings.passwordExpiryHint")}</p></div>
                  <div className="space-y-2"><Label htmlFor="historyCount">{t("settings.securitySettings.passwordHistory")}</Label><Input id="historyCount" name="historyCount" type="number" min={0} max={24} defaultValue={policy.history_count} /><p className="text-xs text-[var(--text-secondary)]">{t("settings.securitySettings.passwordHistoryHint")}</p></div>
                  <div className="space-y-2"><Label htmlFor="lockoutThreshold">{t("settings.securitySettings.lockoutAfter")}</Label><Input id="lockoutThreshold" name="lockoutThreshold" type="number" min={0} max={100} defaultValue={policy.lockout_threshold} /><p className="text-xs text-[var(--text-secondary)]">{t("settings.securitySettings.lockoutAfterHint")}</p></div>
                  <div className="space-y-2"><Label htmlFor="lockoutDurationMins">{t("settings.securitySettings.lockoutDurationMins")}</Label><Input id="lockoutDurationMins" name="lockoutDurationMins" type="number" min={1} max={1440} defaultValue={policy.lockout_duration_mins} /></div>
                </div>

                <div className="space-y-2">
                  <h4 className="text-sm font-medium">{t("settings.securitySettings.characterRequirements")}</h4>
                  <div className="grid gap-0 md:grid-cols-2">
                    <div className="flex min-h-[48px] items-center justify-between gap-4">
                      <Label htmlFor="requireUppercase">{t("settings.securitySettings.requireUppercase")}</Label>
                      <div className="shrink-0">
                        <Switch id="requireUppercase" defaultChecked={policy.require_uppercase} onCheckedChange={(checked: boolean) => syncHiddenBooleanField("requireUppercase-hidden", checked)} />
                        <input id="requireUppercase-hidden" type="hidden" name="requireUppercase" value={policy.require_uppercase ? "true" : "false"} />
                      </div>
                    </div>
                    <div className="flex min-h-[48px] items-center justify-between gap-4">
                      <Label htmlFor="requireLowercase">{t("settings.securitySettings.requireLowercase")}</Label>
                      <div className="shrink-0">
                        <Switch id="requireLowercase" defaultChecked={policy.require_lowercase} onCheckedChange={(checked: boolean) => syncHiddenBooleanField("requireLowercase-hidden", checked)} />
                        <input id="requireLowercase-hidden" type="hidden" name="requireLowercase" value={policy.require_lowercase ? "true" : "false"} />
                      </div>
                    </div>
                    <div className="flex min-h-[48px] items-center justify-between gap-4">
                      <Label htmlFor="requireNumbers">{t("settings.securitySettings.requireNumbers")}</Label>
                      <div className="shrink-0">
                        <Switch id="requireNumbers" defaultChecked={policy.require_numbers} onCheckedChange={(checked: boolean) => syncHiddenBooleanField("requireNumbers-hidden", checked)} />
                        <input id="requireNumbers-hidden" type="hidden" name="requireNumbers" value={policy.require_numbers ? "true" : "false"} />
                      </div>
                    </div>
                    <div className="flex min-h-[48px] items-center justify-between gap-4">
                      <Label htmlFor="requireSymbols">{t("settings.securitySettings.requireSymbols")}</Label>
                      <div className="shrink-0">
                        <Switch id="requireSymbols" defaultChecked={policy.require_symbols} onCheckedChange={(checked: boolean) => syncHiddenBooleanField("requireSymbols-hidden", checked)} />
                        <input id="requireSymbols-hidden" type="hidden" name="requireSymbols" value={policy.require_symbols ? "true" : "false"} />
                      </div>
                    </div>
                  </div>
                </div>

                {actionData?.error && <div className="rounded-md bg-red-50 p-3 text-sm text-[var(--accent-red)]">{actionData.error}</div>}
                {actionData?.success && <div className="rounded-md bg-[var(--accent-green)]/10 p-3 text-sm text-[var(--accent-green)]">{actionData.message}</div>}

                <div className={actionBarClassName}>
                  <Button type="submit" disabled={isSubmitting}>{isSubmitting ? t("settings.securitySettings.saving") : t("settings.securitySettings.savePolicy")}</Button>
                </div>
              </Form>
            )}
          </div>
        </CardContent>
      </Card>
      {/* CAPTCHA Status (read-only — configured via environment variables) */}
      <Card>
        <CardHeader>
          <SettingsSectionHeading
            title={t("settings.securitySettings.captchaTitle")}
            description={t("settings.securitySettings.captchaDescription")}
          />
        </CardHeader>
        <CardContent>
          <div className="text-sm text-[var(--text-secondary)] space-y-2">
            <p>{t("settings.securitySettings.captchaEnvNote")}</p>
            <div className="grid grid-cols-2 gap-2 max-w-sm">
              <span className="font-medium">{t("settings.securitySettings.captchaStatus")}</span>
              <span>CAPTCHA_ENABLED</span>
              <span className="font-medium">{t("settings.securitySettings.captchaProvider")}</span>
              <span>CAPTCHA_PROVIDER</span>
              <span className="font-medium">{t("settings.securitySettings.captchaMode")}</span>
              <span>CAPTCHA_MODE</span>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
