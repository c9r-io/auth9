import type { ReactNode } from "react";
import { LanguageSwitcher } from "~/components/LanguageSwitcher";
import { ThemeToggle } from "~/components/ThemeToggle";
import { cn } from "~/lib/utils";

interface AuthPageShellProps {
  children: ReactNode;
  className?: string;
}

export function AuthPageShell({ children, className }: AuthPageShellProps) {
  return (
    <>
      <div className="fixed top-6 right-6 z-20 flex items-center gap-3">
        <LanguageSwitcher />
        <ThemeToggle />
      </div>

      <div className={cn("auth-page-shell min-h-screen flex items-center justify-center px-4 sm:px-6 relative", className)}>
        <div className="page-backdrop" />
        <div className="relative z-10 flex w-full justify-center">
          {children}
        </div>
      </div>
    </>
  );
}
