import { useFetcher, useRevalidator } from "react-router";
import { useLocale, useI18n, type AppLocale } from "~/i18n";
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
} from "~/components/ui/dropdown-menu";

export function LanguageSwitcher() {
  const { t } = useI18n();
  const { locale, setLocale } = useLocale();
  const revalidator = useRevalidator();
  const fetcher = useFetcher();

  const handleChange = async (value: string) => {
    const nextLocale = value as AppLocale;
    if (nextLocale === locale) return;
    await setLocale(nextLocale);
    fetcher.submit({ locale: nextLocale }, { method: "post", action: "/api/locale" });
    revalidator.revalidate();
  };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          className="theme-btn"
          aria-label={t("common.language.switcherLabel")}
        >
          <GlobeIcon />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuRadioGroup value={locale} onValueChange={handleChange}>
          <DropdownMenuRadioItem value="zh-CN">
            {t("common.language.zhCN")}
          </DropdownMenuRadioItem>
          <DropdownMenuRadioItem value="en-US">
            {t("common.language.enUS")}
          </DropdownMenuRadioItem>
          <DropdownMenuRadioItem value="ja">
            {t("common.language.ja")}
          </DropdownMenuRadioItem>
        </DropdownMenuRadioGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function GlobeIcon() {
  return (
    <svg fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18ZM3.6 9h16.8M3.6 15h16.8M11.5 3a17 17 0 0 0 0 18M12.5 3a17 17 0 0 1 0 18"
      />
    </svg>
  );
}
