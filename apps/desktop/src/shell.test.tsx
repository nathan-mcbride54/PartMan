import {
  foregroundClass,
  pairClass,
  pairs,
  shapeClass,
} from "@partman/design-tokens";
import type { TextContrastPair } from "@partman/design-tokens";
import { PartManShell } from "@partman/ui";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, expectTypeOf, test, vi } from "vitest";

import {
  formatByteSize,
  formatExactBytes,
  formatIecBytes,
} from "./format-bytes";
import { App } from "./App";
import { workspacePreview } from "./preview";
import { strings } from "./strings";

describe("UI-013 byte formatting", () => {
  const boundaryCases: readonly (readonly [bigint, string])[] = [
    [0n, "0 B"],
    [1_023n, "1023 B"],
    [1_024n, "1.0 KiB"],
    [1_048_576n, "1.0 MiB"],
    [1_073_741_824n, "1.0 GiB"],
    [1_099_511_627_776n, "1.0 TiB"],
  ];

  test.each(boundaryCases)("formats %s bytes at IEC boundaries", (bytes, expected) => {
    expect(formatIecBytes(bytes)).toBe(expected);
  });

  test("rounds IEC values to one decimal with integer arithmetic", () => {
    const mebibyte = 1_048_576n;
    const roundsUpAt = (21n * mebibyte + 19n) / 20n;

    expect(formatIecBytes(roundsUpAt - 1n)).toBe("1.0 MiB");
    expect(formatIecBytes(roundsUpAt)).toBe("1.1 MiB");
    expect(formatIecBytes(1_073_741_824n - 1n)).toBe("1.0 GiB");
  });

  test("groups exact bytes without converting through Number", () => {
    expect(formatExactBytes(1_000_204_886_016n)).toBe(
      "1,000,204,886,016 B",
    );
    expect(formatByteSize(272_629_760n)).toEqual({
      displaySize: "260.0 MiB",
      exactBytes: "272,629,760 B",
    });
  });

  test("preserves the full canonical unsigned 64-bit boundary", () => {
    const maximumUnsigned64 = 18_446_744_073_709_551_615n;

    expect(formatIecBytes(maximumUnsigned64)).toBe("16.0 EiB");
    expect(formatExactBytes(maximumUnsigned64)).toBe(
      "18,446,744,073,709,551,615 B",
    );
  });

  test("uses the requested locale for decimal and grouping punctuation", () => {
    expect(formatIecBytes(1_048_576n, "de-DE")).toBe("1,0 MiB");
    expect(formatExactBytes(1_000_204_886_016n, "de-DE")).toBe(
      "1.000.204.886.016 B",
    );
  });

  test("rejects values outside the unsigned canonical model", () => {
    expect(() => formatIecBytes(-1n)).toThrow(RangeError);
    expect(() => formatExactBytes(-1n)).toThrow(RangeError);
  });
});

describe("WP-030 desktop shell", () => {
  test("text class helpers reject UI-only contrast pairs at the type boundary", () => {
    expectTypeOf(pairClass)
      .parameter(0)
      .toEqualTypeOf<TextContrastPair>();
    expectTypeOf(foregroundClass)
      .parameter(0)
      .toEqualTypeOf<TextContrastPair>();
    expectTypeOf(pairs.textPrimaryOnSurfaceBaseText).toMatchTypeOf<TextContrastPair>();
    expectTypeOf(
      pairs.entityFreeSpaceOnSurfaceBaseUi,
    ).not.toMatchTypeOf<TextContrastPair>();
  });

  test("defaults to dark and exposes every UI-001 theme choice", () => {
    const markup = renderToStaticMarkup(<App />);

    expect(markup).toContain(`<option value="dark" selected="">Dark</option>`);
    for (const label of Object.values(strings.themeOptions)) {
      expect(markup).toContain(label);
    }
  });

  test("renders every UI-002 workspace region with an explicit preview warning", () => {
    const markup = renderToStaticMarkup(
      <PartManShell
        preview={workspacePreview}
        strings={strings}
        theme="dark"
        onThemeChange={() => undefined}
      />,
    );

    expect(markup).toContain(strings.previewNotice);
    expect(markup).toContain(strings.previewExplanation);
    expect(markup).toContain(`id="device-rail-heading"`);
    expect(markup).toContain(`id="topology-heading"`);
    expect(markup).toContain(`id="inspector-heading"`);
    expect(markup).toContain(`id="plan-heading"`);
    expect(markup).not.toContain("Apply</button>");
  });

  test("the preview vocabulary contains every storage entity role", () => {
    const roles = new Set(
      workspacePreview.devices.flatMap((device) =>
        device.nodes.map((node) => node.role),
      ),
    );
    roles.add("entity.device");

    expect(roles).toEqual(
      new Set([
        "entity.device",
        "entity.partition",
        "entity.container",
        "entity.volume",
        "entity.encryption",
        "entity.filesystem",
        "entity.mount",
        "entity.freeSpace",
      ]),
    );
  });

  test("exact byte values accompany every IEC topology size", () => {
    for (const device of workspacePreview.devices) {
      const deviceSize = strings.formatByteSize(device.sizeBytes);
      expect(deviceSize.displaySize).toMatch(
        /^(?:\d+|\d+\.\d) (?:[KMGTPE]i)?B$/u,
      );
      expect(deviceSize.displaySize).not.toMatch(
        /\b(?:KB|MB|GB|TB|PB|EB)\b/u,
      );
      expect(deviceSize.exactBytes).toMatch(/^[0-9,]+ B$/u);
      for (const node of device.nodes) {
        const nodeSize = strings.formatByteSize(node.sizeBytes);
        expect(nodeSize.displaySize).toMatch(
          /^(?:\d+|\d+\.\d) (?:[KMGTPE]i)?B$/u,
        );
        expect(nodeSize.displaySize).not.toMatch(
          /\b(?:KB|MB|GB|TB|PB|EB)\b/u,
        );
        expect(nodeSize.exactBytes).toMatch(/^[0-9,]+ B$/u);
      }
    }
  });

  test("uses the expected IEC and exact values for every synthetic size", () => {
    const firstDevice = workspacePreview.devices[0];
    const secondDevice = workspacePreview.devices[1];

    expect(strings.formatByteSize(firstDevice?.sizeBytes ?? -1n)).toEqual({
      displaySize: "931.5 GiB",
      exactBytes: "1,000,204,886,016 B",
    });
    expect(
      firstDevice?.nodes.map(({ sizeBytes }) => {
        const size = strings.formatByteSize(sizeBytes);
        return [size.displaySize, size.exactBytes];
      }),
    ).toEqual([
      ["260.0 MiB", "272,629,760 B"],
      ["698.2 GiB", "749,731,708,928 B"],
      ["1.0 GiB", "1,073,741,824 B"],
      ["232.0 GiB", "249,126,805,504 B"],
      ["698.2 GiB", "749,731,708,928 B"],
      ["698.2 GiB", "749,731,708,928 B"],
      ["698.2 GiB", "749,731,708,928 B"],
    ]);
    expect(strings.formatByteSize(secondDevice?.sizeBytes ?? -1n)).toEqual({
      displaySize: "465.8 GiB",
      exactBytes: "500,107,862,016 B",
    });
    expect(
      secondDevice?.nodes.map(({ sizeBytes }) => {
        const size = strings.formatByteSize(sizeBytes);
        return [size.displaySize, size.exactBytes];
      }),
    ).toEqual([
      ["465.8 GiB", "500,106,813,440 B"],
      ["465.8 GiB", "500,106,813,440 B"],
      ["391.2 GiB", "420,000,000,000 B"],
      ["391.2 GiB", "420,000,000,000 B"],
    ]);
  });

  test("externalizes device and topology count labels", () => {
    expect(strings.deviceSizeLabel).toBe("Device display size");
    expect(strings.deviceExactBytesLabel).toBe("Device exact bytes");
    expect(strings.deviceCountLabel(1)).toBe("1 synthetic device");
    expect(strings.deviceCountLabel(2)).toBe("2 synthetic devices");
    expect(strings.topologyItemCountLabel(1)).toBe("1 topology item");
    expect(strings.topologyItemCountLabel(2)).toBe("2 topology items");
    expect(strings.healthOptions).toEqual({
      healthy: "Healthy",
      attention: "Needs attention",
      unknown: "Unknown",
    });
    expect(strings.meaningLabels["entity.freeSpace"]).toBe("Free space");
    expect(strings.meaningLabels["severity.dataMoving"]).toBe("Data-moving");
  });

  test("renders CSP-safe topology weights and visible selection state", () => {
    const firstDevice = workspacePreview.devices[0];
    if (!firstDevice) {
      throw new Error("the synthetic preview must include its first device");
    }
    const markup = renderToStaticMarkup(
      <PartManShell
        preview={workspacePreview}
        strings={strings}
        theme="dark"
        onThemeChange={() => undefined}
      />,
    );

    expect(markup).not.toContain(" style=");
    expect(markup).toContain(`data-weight="1"`);
    expect(markup).toContain(`data-weight="17"`);
    expect(markup).toContain(strings.selectedLabel);
    expect(markup).toContain(
      strings.formatByteSize(firstDevice.sizeBytes).exactBytes,
    );
  });

  test("keeps every byte-valued fact as bigint until the inspector formats it", () => {
    const byteFacts = workspacePreview.devices.flatMap((device) =>
      device.nodes.flatMap((node) =>
        node.facts.filter((fact) => fact.kind === "byteSize"),
      ),
    );
    const textFacts = workspacePreview.devices.flatMap((device) =>
      device.nodes.flatMap((node) =>
        node.facts.filter((fact) => fact.kind === "text"),
      ),
    );

    expect(byteFacts.length).toBeGreaterThan(0);
    expect(byteFacts.every((fact) => typeof fact.bytes === "bigint")).toBe(
      true,
    );
    expect(
      textFacts.some((fact) => /^[0-9,]+ (?:[KMGTPE]i)?B$/u.test(fact.value)),
    ).toBe(false);

    const markup = renderToStaticMarkup(
      <PartManShell
        preview={workspacePreview}
        strings={strings}
        theme="dark"
        onThemeChange={() => undefined}
      />,
    );
    expect(markup).toContain("Start offset, exact bytes");
    expect(markup).toContain("1.0 MiB");
    expect(markup).toContain("1,048,576 B");
  });

  test("gets semantic labels from the externalized English catalogue", () => {
    const localizedStrings = {
      ...strings,
      meaningLabels: {
        ...strings.meaningLabels,
        "entity.partition": "Localized partition",
      },
    };
    const markup = renderToStaticMarkup(
      <PartManShell
        preview={workspacePreview}
        strings={localizedStrings}
        theme="dark"
        onThemeChange={() => undefined}
      />,
    );

    expect(markup).toContain("Localized partition");
  });

  test("renders topology shapes through the generated role mapping", () => {
    const reversedPreview = {
      ...workspacePreview,
      devices: [...workspacePreview.devices].reverse(),
    };
    const markup = renderToStaticMarkup(
      <PartManShell
        preview={reversedPreview}
        strings={strings}
        theme="dark"
        onThemeChange={() => undefined}
      />,
    );

    expect(markup).toContain(shapeClass("entity.device"));
    expect(markup).toContain(shapeClass("entity.container"));
    expect(markup).toContain(shapeClass("entity.volume"));
    expect(shapeClass("entity.container")).not.toBe(
      shapeClass("entity.volume"),
    );
  });

  test("the pending-plan drawer starts collapsed in a narrow workspace", () => {
    vi.stubGlobal("matchMedia", () => ({ matches: true }));
    try {
      const markup = renderToStaticMarkup(
        <PartManShell
          preview={workspacePreview}
          strings={strings}
          theme="dark"
          onThemeChange={() => undefined}
        />,
      );

      expect(markup).toContain(`data-plan-open="false"`);
      expect(markup).toContain(strings.openPlanLabel);
      expect(markup).toContain(`aria-controls="plan-drawer-content"`);
      expect(markup).toContain(`id="plan-drawer-content"`);
      expect(markup).toContain(`hidden=""`);
      expect(markup).toContain(workspacePreview.plan.title);
    } finally {
      vi.unstubAllGlobals();
    }
  });
});
