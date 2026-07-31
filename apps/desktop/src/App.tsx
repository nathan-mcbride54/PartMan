import { PartManShell } from "@partman/ui";
import type { ThemeChoice } from "@partman/ui";
import { useEffect, useState } from "react";

import { workspacePreview } from "./preview";
import { strings } from "./strings";

export function App() {
  const [theme, setTheme] = useState<ThemeChoice>("dark");

  useEffect(() => {
    if (theme === "system") {
      delete document.documentElement.dataset.theme;
      return;
    }
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  return (
    <PartManShell
      preview={workspacePreview}
      strings={strings}
      theme={theme}
      onThemeChange={setTheme}
    />
  );
}
