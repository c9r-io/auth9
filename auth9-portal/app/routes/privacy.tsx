import type { MetaFunction } from "react-router";
import { PublicPageLayout } from "~/components/PublicPageLayout";
import { useI18n } from "~/i18n";
import { buildMeta, resolveMetaLocale } from "~/i18n/meta";

export const meta: MetaFunction = ({ matches }) => {
  return buildMeta(
    resolveMetaLocale(matches),
    "legal.privacy.metaTitle",
    "legal.privacy.metaDescription"
  );
};

const SECTIONS = [
  "dataCollection",
  "dataUsage",
  "dataSharing",
  "cookies",
  "security",
  "contact",
] as const;

export default function PrivacyPage() {
  const { t } = useI18n();

  return (
    <PublicPageLayout title={t("legal.privacy.title")}>
      <p className="text-sm mb-8 text-[var(--text-tertiary)]">
        {t("legal.privacy.lastUpdated")}
      </p>
      {SECTIONS.map((key) => (
        <section key={key} className="mb-8">
          <h2>{t(`legal.privacy.sections.${key}.title`)}</h2>
          <p>{t(`legal.privacy.sections.${key}.body`)}</p>
        </section>
      ))}
    </PublicPageLayout>
  );
}
