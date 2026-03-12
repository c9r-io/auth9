import { Link } from "react-router";
import { LanguageSwitcher } from "~/components/LanguageSwitcher";
import { ThemeToggle } from "~/components/ThemeToggle";
import { useI18n } from "~/i18n";

interface PublicPageLayoutProps {
  children: React.ReactNode;
  title: string;
}

export function PublicPageLayout({ children, title }: PublicPageLayoutProps) {
  const { t } = useI18n();

  return (
    <div className="min-h-screen relative">
      <div className="page-backdrop" />

      <header className="fixed top-0 left-0 right-0 z-50 liquid-glass border-b border-[var(--glass-border-subtle)]">
        <div className="max-w-3xl mx-auto px-6 h-16 flex items-center justify-between">
          <Link to="/" className="flex items-center gap-2">
            <div className="logo-icon">A9</div>
            <span className="text-xl font-semibold text-[var(--text-primary)]">
              Auth9
            </span>
          </Link>
          <div className="flex items-center gap-4">
            <LanguageSwitcher />
            <ThemeToggle />
          </div>
        </div>
      </header>

      <main className="pt-32 pb-20 px-6 relative z-10">
        <div className="max-w-3xl mx-auto">
          <h1 className="text-3xl md:text-4xl font-bold tracking-tight text-[var(--text-primary)] mb-8">
            {title}
          </h1>
          <div className="prose-glass">{children}</div>
        </div>
      </main>

      <footer className="border-t border-[var(--glass-border-subtle)] py-12 px-6 relative z-10">
        <div className="max-w-3xl mx-auto flex items-center justify-between">
          <p className="text-[var(--text-tertiary)] text-sm">
            {t("home.footer.copyright")}
          </p>
          <div className="flex items-center gap-6">
            <Link
              to="/privacy"
              className="text-[var(--text-tertiary)] hover:text-[var(--text-primary)] text-sm transition-colors"
            >
              {t("home.footer.privacy")}
            </Link>
            <Link
              to="/terms"
              className="text-[var(--text-tertiary)] hover:text-[var(--text-primary)] text-sm transition-colors"
            >
              {t("home.footer.terms")}
            </Link>
          </div>
        </div>
      </footer>
    </div>
  );
}
