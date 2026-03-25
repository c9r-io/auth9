import type { LoaderFunctionArgs, MetaFunction } from "react-router";
import { Link, useLoaderData } from "react-router";
import { Button } from "~/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card";
import { useI18n } from "~/i18n";
import { buildMeta, resolveMetaLocale } from "~/i18n/meta";
import { translate } from "~/i18n/translate";
import { resolveLocale } from "~/services/locale.server";
import { riskApi } from "~/services/api";
import { getAccessToken } from "~/services/session.server";

export const meta: MetaFunction = ({ matches }) => buildMeta(resolveMetaLocale(matches), "securityRisk.metaTitle");

export async function loader({ request }: LoaderFunctionArgs) {
  const locale = await resolveLocale(request);
  const accessToken = await getAccessToken(request);

  try {
    const response = await riskApi.getRiskPolicy(accessToken || undefined);
    return { riskPolicy: response.data, error: null };
  } catch {
    return {
      riskPolicy: null,
      error: translate(locale, "riskPolicy.saveFailed"),
    };
  }
}

function getRiskLevelColor(level: string) {
  switch (level) {
    case "critical":
      return "border-l-red-500 bg-red-50";
    case "high":
      return "border-l-orange-500 bg-orange-50";
    case "medium":
      return "border-l-yellow-500 bg-yellow-50";
    case "low":
      return "border-l-green-500 bg-green-50";
    default:
      return "border-l-gray-500 bg-gray-50";
  }
}

export default function SecurityRiskPage() {
  const { riskPolicy, error } = useLoaderData<typeof loader>();
  const { t } = useI18n();

  const riskLevels = [
    { key: "low", label: t("securityRisk.lowRisk"), color: getRiskLevelColor("low") },
    { key: "medium", label: t("securityRisk.mediumRisk"), color: getRiskLevelColor("medium") },
    { key: "high", label: t("securityRisk.highRisk"), color: getRiskLevelColor("high") },
    { key: "critical", label: t("securityRisk.criticalRisk"), color: getRiskLevelColor("critical") },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">{t("securityRisk.title")}</h1>
        <p className="text-[var(--text-secondary)]">{t("riskPolicy.description")}</p>
      </div>

      {error && <div className="text-sm text-[var(--accent-red)] bg-red-50 p-3 rounded-md">{error}</div>}

      {riskPolicy && (
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">{t("securityRisk.currentPolicy")}</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid gap-4 sm:grid-cols-2">
              <div className="rounded-lg border p-4">
                <div className="text-sm text-[var(--text-secondary)]">{t("riskPolicy.mfaThreshold")}</div>
                <div className="mt-1 text-2xl font-bold">{riskPolicy.mfa_threshold}</div>
              </div>
              <div className="rounded-lg border p-4">
                <div className="text-sm text-[var(--text-secondary)]">{t("riskPolicy.blockThreshold")}</div>
                <div className="mt-1 text-2xl font-bold">{riskPolicy.block_threshold}</div>
              </div>
              <div className="rounded-lg border p-4">
                <div className="text-sm text-[var(--text-secondary)]">{t("riskPolicy.notifyAdmin")}</div>
                <div className="mt-1 text-lg font-medium">{riskPolicy.notify_admin ? "Enabled" : "Disabled"}</div>
              </div>
              <div className="rounded-lg border p-4">
                <div className="text-sm text-[var(--text-secondary)]">{t("riskPolicy.autoLockAccount")}</div>
                <div className="mt-1 text-lg font-medium">{riskPolicy.auto_lock_account ? "Enabled" : "Disabled"}</div>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      <Card>
        <CardHeader>
          <CardTitle className="text-lg">{t("securityRisk.riskLevels")}</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {riskLevels.map((level) => (
              <div key={level.key} className={`border-l-4 rounded-md p-3 text-sm ${level.color}`}>
                {level.label}
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      <div className="flex gap-3">
        <Link to="/dashboard/security/alerts">
          <Button variant="outline">{t("securityRisk.viewAlerts")}</Button>
        </Link>
        <Link to="/dashboard/settings/security">
          <Button variant="outline">{t("securityRisk.configurePolicy")}</Button>
        </Link>
      </div>
    </div>
  );
}
