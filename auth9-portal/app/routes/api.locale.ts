import type { ActionFunctionArgs } from "react-router";
import { serializeLocaleCookie } from "~/services/locale.server";
import { SUPPORTED_LOCALES, type AppLocale } from "~/i18n";

export async function action({ request }: ActionFunctionArgs) {
  const formData = await request.formData();
  const locale = formData.get("locale") as string;

  if (!locale || !SUPPORTED_LOCALES.includes(locale as AppLocale)) {
    return new Response("Invalid locale", { status: 400 });
  }

  return new Response(null, {
    status: 204,
    headers: {
      "Set-Cookie": await serializeLocaleCookie(locale as AppLocale),
    },
  });
}
