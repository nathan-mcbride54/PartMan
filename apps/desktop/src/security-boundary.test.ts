import { describe, expect, test } from "vitest";

import packageManifest from "../package.json";
import mainUiCapability from "../src-tauri/capabilities/main-ui.json";
import tauriConfig from "../src-tauri/tauri.conf.json";

describe("WP-030 native shell security boundary", () => {
  test("UI-002 production content is embedded behind an exact least-privilege CSP", () => {
    expect(tauriConfig.build.frontendDist).toBe("../dist");
    expect(tauriConfig.app.security.csp).toStrictEqual({
      "default-src": "'self'",
      "script-src": "'self'",
      "font-src": "'self'",
      "style-src": "'self'",
      "connect-src": "'none'",
      "img-src": "'self'",
      "object-src": "'none'",
      "base-uri": "'none'",
      "frame-src": "'none'",
      "form-action": "'none'",
    });
  });

  test("SAFE-002 grants the main window no native permissions", () => {
    expect(tauriConfig.app.security.capabilities).toStrictEqual(["main-ui"]);
    expect(mainUiCapability).toStrictEqual({
      $schema: "../gen/schemas/desktop-schema.json",
      identifier: "main-ui",
      description: "Read-only presentation capability for PartMan's main window",
      windows: ["main"],
      platforms: ["linux", "macOS", "windows"],
      permissions: [],
    });
  });

  test("the standalone native production gate refuses Cargo lockfile drift", () => {
    expect(packageManifest.scripts["native:build"]).toBe(
      "tauri build --no-bundle -- --locked",
    );
  });
});
