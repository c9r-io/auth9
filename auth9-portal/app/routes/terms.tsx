import type { MetaFunction } from "react-router";
import { PublicPageLayout } from "~/components/PublicPageLayout";
import { useI18n } from "~/i18n";
import { buildMeta, resolveMetaLocale } from "~/i18n/meta";

export const meta: MetaFunction = ({ matches }) => {
  return buildMeta(
    resolveMetaLocale(matches),
    "legal.terms.metaTitle",
    "legal.terms.metaDescription"
  );
};

const SECTIONS = [
  "acceptance",
  "accountResponsibility",
  "usageRules",
  "intellectualProperty",
  "limitationOfLiability",
  "termination",
  "governingLaw",
  "contact",
] as const;

export default function TermsPage() {
  const { t } = useI18n();

  return (
    <PublicPageLayout title={t("legal.terms.title")}>
      <p className="text-sm mb-8 text-[var(--text-tertiary)]">
        {t("legal.terms.lastUpdated")}
      </p>
      {SECTIONS.map((key) => (
        <section key={key} className="mb-8">
          <h2>{t(`legal.terms.sections.${key}.title`)}</h2>
          <p>{t(`legal.terms.sections.${key}.body`)}</p>
        </section>
      ))}
    </PublicPageLayout>
  );
}
