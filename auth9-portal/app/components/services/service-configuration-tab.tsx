import { CopyIcon, EyeClosedIcon, EyeOpenIcon, PlusIcon, TrashIcon, UpdateIcon } from "@radix-ui/react-icons";
import { useState } from "react";
import { Form } from "react-router";
import { Button } from "~/components/ui/button";
import { Card, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "~/components/ui/dialog";
import { Input } from "~/components/ui/input";
import { Label } from "~/components/ui/label";
import { useI18n } from "~/i18n";
import type { Client, Service, ServiceIntegrationInfo } from "~/services/api";
import { copyToClipboard } from "./copyable-value";

interface ServiceConfigurationTabProps {
  actionError?: string | null;
  clients: Client[];
  formatDate: (value: string) => string;
  integration: ServiceIntegrationInfo | null;
  isAddClientOpen: boolean;
  isSubmitting: boolean;
  service: Service;
  onAddClientOpenChange: (open: boolean) => void;
  onDeleteClient: (clientId: string) => void | Promise<void>;
  onRegenerateSecret: (clientId: string) => void | Promise<void>;
}

export function ServiceConfigurationTab({
  actionError,
  clients,
  formatDate,
  integration,
  isAddClientOpen,
  isSubmitting,
  service,
  onAddClientOpenChange,
  onDeleteClient,
  onRegenerateSecret,
}: ServiceConfigurationTabProps) {
  const { t } = useI18n();
  const [copiedField, setCopiedField] = useState<string | null>(null);
  const [revealedSecrets, setRevealedSecrets] = useState<Set<string>>(new Set());

  const toggleReveal = (clientId: string) => {
    setRevealedSecrets((previous) => {
      const next = new Set(previous);
      if (next.has(clientId)) {
        next.delete(clientId);
      } else {
        next.add(clientId);
      }
      return next;
    });
  };

  const handleCopy = async (text: string, fieldName: string) => {
    await copyToClipboard(text);
    setCopiedField(fieldName);
    setTimeout(() => setCopiedField(null), 2000);
  };

  return (
    <div className="grid grid-cols-1 gap-6 md:grid-cols-3">
      <div className="md:col-span-2">
        <Card>
          <CardHeader>
            <CardTitle>{t("services.detail.configurationTitle")}</CardTitle>
            <CardDescription>{t("services.detail.configurationDescription")}</CardDescription>
          </CardHeader>
          <div className="p-6">
            {actionError && (
              <div className="mb-4 rounded-lg border border-[var(--accent-red)]/30 bg-[var(--accent-red)]/10 p-3 text-sm text-[var(--accent-red)]">
                {actionError}
              </div>
            )}
            <Form method="post" className="space-y-4">
              <input type="hidden" name="intent" value="update_service" />
              <div className="space-y-2">
                <Label htmlFor="name">{t("services.serviceName")}</Label>
                <Input id="name" name="name" defaultValue={service.name} required />
              </div>
              <div className="space-y-2">
                <Label htmlFor="base_url">{t("services.baseUrl")}</Label>
                <Input
                  id="base_url"
                  name="base_url"
                  defaultValue={service.base_url}
                  placeholder={t("services.baseUrlPlaceholder")}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="redirect_uris">{t("services.redirectUris")}</Label>
                <Input id="redirect_uris" name="redirect_uris" defaultValue={service.redirect_uris?.join(", ")} />
              </div>
              <div className="space-y-2">
                <Label htmlFor="logout_uris">{t("services.logoutUris")}</Label>
                <Input id="logout_uris" name="logout_uris" defaultValue={service.logout_uris?.join(", ")} />
              </div>
              <div className="flex justify-end pt-4">
                <Button type="submit" disabled={isSubmitting}>
                  {isSubmitting ? t("services.detail.saving") : t("services.detail.saveChanges")}
                </Button>
              </div>
            </Form>
          </div>
        </Card>
      </div>

      <div>
        <Card className="h-full">
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <div className="space-y-1">
              <CardTitle>{t("services.detail.clientsTitle")}</CardTitle>
              <CardDescription>{t("services.detail.clientsDescription")}</CardDescription>
            </div>
            <Dialog open={isAddClientOpen} onOpenChange={onAddClientOpenChange}>
              <DialogTrigger asChild>
                <Button size="sm" variant="outline" title={t("services.detail.createClientTitle")}>
                  <PlusIcon className="h-4 w-4" />
                </Button>
              </DialogTrigger>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>{t("services.detail.createClientTitle")}</DialogTitle>
                  <DialogDescription>{t("services.detail.createClientDescription")}</DialogDescription>
                </DialogHeader>
                <Form method="post" className="space-y-4">
                  <input type="hidden" name="intent" value="create_client" />
                  <div className="space-y-2">
                    <Label htmlFor="client-name">{t("services.detail.clientDescriptionOptional")}</Label>
                    <Input
                      id="client-name"
                      name="name"
                      placeholder={t("services.detail.clientDescriptionPlaceholder")}
                    />
                  </div>
                  <DialogFooter>
                    <Button type="button" variant="outline" onClick={() => onAddClientOpenChange(false)}>
                      {t("common.buttons.cancel")}
                    </Button>
                    <Button type="submit" disabled={isSubmitting}>
                      {t("services.detail.create")}
                    </Button>
                  </DialogFooter>
                </Form>
              </DialogContent>
            </Dialog>
          </CardHeader>
          <div className="p-0">
            <ul className="divide-y divide-[var(--glass-border-subtle)]">
              {clients.map((client) => (
                <li key={client.id} className="p-4 hover:bg-[var(--sidebar-item-hover)]">
                  <div className="mb-2 flex items-start justify-between">
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center gap-2">
                        <code className="truncate font-mono text-sm font-medium text-[var(--text-primary)]">
                          {client.client_id}
                        </code>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-11 w-11 text-[var(--text-tertiary)] hover:text-[var(--text-secondary)] sm:h-8 sm:w-8"
                          onClick={() => handleCopy(client.client_id, `client-${client.id}`)}
                          title={t("services.detail.copyClientId")}
                        >
                          {copiedField === `client-${client.id}` ? (
                            <span className="text-xs text-[var(--accent-green)]">&#10003;</span>
                          ) : (
                            <CopyIcon className="h-3 w-3" />
                          )}
                        </Button>
                      </div>
                      {(() => {
                        const integrationClient = integration?.clients.find(
                          (c) => c.client_id === client.client_id
                        );
                        return (
                          <div className="mt-1.5">
                            <span className="text-[11px] text-[var(--text-tertiary)]">
                              {t("services.detail.clientSecret")}
                            </span>
                            {integrationClient?.client_secret ? (
                              <div className="flex min-w-0 items-center gap-1">
                                <code className="min-w-0 flex-1 select-all break-all whitespace-normal font-mono text-xs text-[var(--text-primary)] [word-break:break-all]">
                                  {revealedSecrets.has(client.client_id)
                                    ? integrationClient.client_secret
                                    : "••••••••••••••••••••••••"}
                                </code>
                                <Button
                                  variant="ghost"
                                  size="icon"
                                  className="h-11 w-11 shrink-0 text-[var(--text-tertiary)] hover:text-[var(--text-secondary)] sm:h-6 sm:w-6"
                                  onClick={() => toggleReveal(client.client_id)}
                                  title={
                                    revealedSecrets.has(client.client_id)
                                      ? t("services.integration.hide")
                                      : t("services.integration.reveal")
                                  }
                                >
                                  {revealedSecrets.has(client.client_id) ? (
                                    <EyeClosedIcon className="h-3 w-3" />
                                  ) : (
                                    <EyeOpenIcon className="h-3 w-3" />
                                  )}
                                </Button>
                                <Button
                                  variant="ghost"
                                  size="icon"
                                  className="h-11 w-11 shrink-0 text-[var(--text-tertiary)] hover:text-[var(--text-secondary)] sm:h-6 sm:w-6"
                                  onClick={() => handleCopy(integrationClient.client_secret!, `secret-${client.id}`)}
                                  title={t("services.integration.copySecret")}
                                >
                                  {copiedField === `secret-${client.id}` ? (
                                    <span className="text-xs text-[var(--accent-green)]">&#10003;</span>
                                  ) : (
                                    <CopyIcon className="h-3 w-3" />
                                  )}
                                </Button>
                              </div>
                            ) : (
                              <div className="text-xs italic text-[var(--text-secondary)]">
                                {t("services.integration.clientSecretUnavailable")}
                              </div>
                            )}
                          </div>
                        );
                      })()}
                      <div className="mt-1 text-xs text-[var(--text-secondary)]">
                        {client.name || t("services.detail.noDescription")}
                      </div>
                      <div className="mt-0.5 text-xs text-[var(--text-tertiary)]">
                        {t("services.detail.createdOn", { date: formatDate(client.created_at) })}
                      </div>
                    </div>
                  </div>
                  <div className="mt-2 flex items-center gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      className="h-7 text-xs"
                      onClick={() => onRegenerateSecret(client.client_id)}
                    >
                      <UpdateIcon className="mr-1 h-3 w-3" />
                      {t("services.detail.regenerate")}
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-7 text-xs text-[var(--accent-red)] hover:bg-[var(--accent-red)]/10 hover:text-[var(--accent-red)]"
                      onClick={() => onDeleteClient(client.client_id)}
                    >
                      <TrashIcon className="mr-1 h-3 w-3" />
                      {t("common.buttons.delete")}
                    </Button>
                  </div>
                </li>
              ))}
              {clients.length === 0 && (
                <li className="p-4 text-center text-sm text-[var(--text-secondary)]">{t("services.detail.noClients")}</li>
              )}
            </ul>
          </div>
        </Card>
      </div>
    </div>
  );
}
