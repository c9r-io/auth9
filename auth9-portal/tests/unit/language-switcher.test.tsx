import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createRoutesStub } from "react-router";
import { LanguageSwitcher } from "~/components/LanguageSwitcher";

function renderSwitcher() {
  const RoutesStub = createRoutesStub([
    {
      path: "/",
      Component: () => <LanguageSwitcher />,
    },
  ]);
  return render(<RoutesStub initialEntries={["/"]} />);
}

describe("LanguageSwitcher", () => {
  it("renders a trigger button with aria-label", async () => {
    renderSwitcher();
    const button = await screen.findByRole("button", {
      name: /switch language|切换语言|言語を切り替え/i,
    });
    expect(button).toBeInTheDocument();
  });

  it("renders all three language options when opened", async () => {
    const user = userEvent.setup();
    renderSwitcher();

    const trigger = await screen.findByRole("button", {
      name: /switch language|切换语言|言語を切り替え/i,
    });
    await user.click(trigger);

    const items = await screen.findAllByRole("menuitemradio");
    expect(items).toHaveLength(3);
  });

  it("displays localized language names when opened", async () => {
    const user = userEvent.setup();
    renderSwitcher();

    const trigger = await screen.findByRole("button", {
      name: /switch language|切换语言|言語を切り替え/i,
    });
    await user.click(trigger);

    await screen.findAllByRole("menuitemradio");
    expect(screen.getByText("简体中文")).toBeInTheDocument();
    expect(screen.getByText("English")).toBeInTheDocument();
    expect(screen.getByText("日本語")).toBeInTheDocument();
  });

  it("allows changing language via radio item click", async () => {
    const user = userEvent.setup();
    renderSwitcher();

    const trigger = await screen.findByRole("button", {
      name: /switch language|切换语言|言語を切り替え/i,
    });
    await user.click(trigger);

    const jaItem = await screen.findByText("日本語");
    await user.click(jaItem);

    // After selection, menu closes; re-open to verify
    await user.click(trigger);
    const items = await screen.findAllByRole("menuitemradio");
    const jaRadio = items.find((item) => item.textContent?.includes("日本語"));
    expect(jaRadio).toHaveAttribute("aria-checked", "true");
  });
});
