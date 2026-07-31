// @vitest-environment happy-dom

import {
  cleanup,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, test, vi } from "vitest";

import { pairClass, pairs } from "@partman/design-tokens";
import { PartManShell } from "@partman/ui";

import { App } from "./App";
import { workspacePreview } from "./preview";
import { strings } from "./strings";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  delete document.documentElement.dataset.theme;
});

describe("WP-030 desktop interactions", () => {
  test("keeps selectable storage buttons on the audited base surface", () => {
    render(
      <PartManShell
        preview={workspacePreview}
        strings={strings}
        theme="dark"
        onThemeChange={() => undefined}
      />,
    );

    const auditedBase = pairClass(pairs.textPrimaryOnSurfaceBaseText);
    const device = screen.getByRole("button", { name: /System NVMe/u });
    const topologyNode = screen.getByRole("button", {
      name: /Partition EFI /u,
    });

    expect(device.classList.contains(auditedBase)).toBe(true);
    expect(topologyNode.classList.contains(auditedBase)).toBe(true);
  });

  test("changes every UI-001 theme through the native select", async () => {
    const user = userEvent.setup();
    render(<App />);

    const theme = screen.getByRole("combobox", {
      name: strings.themeLabel,
    }) as HTMLSelectElement;
    expect(theme.value).toBe("dark");
    await waitFor(() => {
      expect(document.documentElement.dataset.theme).toBe("dark");
    });

    await user.selectOptions(theme, "high-contrast");
    expect(theme.value).toBe("high-contrast");
    expect(document.documentElement.dataset.theme).toBe("high-contrast");

    await user.selectOptions(theme, "light");
    expect(document.documentElement.dataset.theme).toBe("light");

    await user.selectOptions(theme, "system");
    expect(document.documentElement.dataset.theme).toBeUndefined();
  });

  test("updates topology, inspector, and ARIA selection state", async () => {
    const user = userEvent.setup();
    render(
      <PartManShell
        preview={workspacePreview}
        strings={strings}
        theme="dark"
        onThemeChange={() => undefined}
      />,
    );

    const external = screen.getByRole("button", {
      name: /External SSD/u,
    });
    await user.click(external);
    expect(external.getAttribute("aria-current")).toBe("true");

    const inspector = screen.getByRole("complementary", {
      name: strings.inspectorHeading,
    });
    expect(within(inspector).getByText("APFS store")).toBeTruthy();
    expect(within(inspector).getByText("Unknown")).toBeTruthy();

    const container = screen.getByRole("button", {
      name: /Container APFS container/u,
    });
    await user.click(container);
    expect(container.getAttribute("aria-pressed")).toBe("true");
    expect(document.activeElement).toBe(container);
    expect(within(inspector).getByText("APFS container")).toBeTruthy();
    expect(within(inspector).getByText("Container role")).toBeTruthy();
  });

  test("supports keyboard-only device selection in document order", async () => {
    const user = userEvent.setup();
    render(
      <PartManShell
        preview={workspacePreview}
        strings={strings}
        theme="dark"
        onThemeChange={() => undefined}
      />,
    );

    const theme = screen.getByRole("combobox", {
      name: strings.themeLabel,
    });
    const system = screen.getByRole("button", { name: /System NVMe/u });
    const external = screen.getByRole("button", {
      name: /External SSD/u,
    });

    await user.tab();
    expect(document.activeElement).toBe(theme);
    await user.tab();
    expect(document.activeElement).toBe(system);
    await user.tab();
    expect(document.activeElement).toBe(external);

    await user.keyboard("{Enter}");
    expect(external.getAttribute("aria-current")).toBe("true");
  });

  test("keeps the collapsed drawer relationship valid and toggles it", async () => {
    vi.stubGlobal("matchMedia", () => ({ matches: true }));
    const user = userEvent.setup();
    render(
      <PartManShell
        preview={workspacePreview}
        strings={strings}
        theme="dark"
        onThemeChange={() => undefined}
      />,
    );

    const toggle = screen.getByRole("button", {
      name: /Pending plan Open drawer/u,
    });
    const content = document.getElementById("plan-drawer-content");
    if (!(content instanceof HTMLElement)) {
      throw new Error("the controlled plan drawer content must exist");
    }

    expect(toggle.getAttribute("aria-controls")).toBe(content.id);
    expect(toggle.getAttribute("aria-expanded")).toBe("false");
    expect(content.hidden).toBe(true);

    await user.click(toggle);
    expect(toggle.getAttribute("aria-expanded")).toBe("true");
    expect(content.hidden).toBe(false);
    expect(document.activeElement).toBe(toggle);

    await user.click(toggle);
    expect(toggle.getAttribute("aria-expanded")).toBe("false");
    expect(content.hidden).toBe(true);
  });
});
