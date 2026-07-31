import {
  borderClass,
  foregroundClass,
  outlineClass,
  pairClass,
  pairs,
  roleClass,
  shapeClass,
} from "@partman/design-tokens";
import { useMemo, useState } from "react";

import { DeviceRail } from "./DeviceRail";
import { Inspector } from "./Inspector";
import { PlanDrawer } from "./PlanDrawer";
import { TopologyMap } from "./TopologyMap";
import type {
  ThemeChoice,
  UiStrings,
  WorkspacePreview,
} from "./types";

interface PartManShellProps {
  readonly preview: WorkspacePreview;
  readonly strings: UiStrings;
  readonly theme: ThemeChoice;
  readonly onThemeChange: (theme: ThemeChoice) => void;
}

function planStartsOpen() {
  return (
    typeof globalThis.matchMedia !== "function" ||
    !globalThis.matchMedia("(max-width: 62rem)").matches
  );
}

export function PartManShell({
  preview,
  strings,
  theme,
  onThemeChange,
}: PartManShellProps) {
  const firstDevice = preview.devices[0];
  if (!firstDevice) {
    throw new Error("the shell preview requires at least one device");
  }

  const [selectedDeviceId, setSelectedDeviceId] = useState(firstDevice.id);
  const selectedDevice =
    preview.devices.find((device) => device.id === selectedDeviceId) ??
    firstDevice;
  const firstNode = selectedDevice.nodes[0];
  if (!firstNode) {
    throw new Error("the shell preview requires at least one topology node");
  }

  const [selectedNodeByDevice, setSelectedNodeByDevice] = useState<
    Readonly<Record<string, string>>
  >({});
  const selectedNode = useMemo(() => {
    const requested = selectedNodeByDevice[selectedDevice.id];
    return (
      selectedDevice.nodes.find((node) => node.id === requested) ?? firstNode
    );
  }, [firstNode, selectedDevice, selectedNodeByDevice]);
  const [planOpen, setPlanOpen] = useState(planStartsOpen);

  return (
    <div
      className={`shell ${pairClass(pairs.textPrimaryOnSurfaceSunkenText)}`}
      data-plan-open={planOpen}
    >
      <header
        className={`app-header ${pairClass(pairs.textPrimaryOnSurfaceBaseText)} ${borderClass(pairs.borderDefaultOnSurfaceBaseUi)}`}
      >
        <div className="brand">
          <span
            className={`brand__mark ${pairClass(pairs.textPrimaryOnSurfaceOverlayText)}`}
            aria-hidden="true"
          >
            P
          </span>
          <div>
            <strong>{strings.appName}</strong>
            <span
              className={foregroundClass(pairs.textMutedOnSurfaceBaseText)}
            >
              {strings.productQualifier}
            </span>
          </div>
        </div>
        <div
          className={`source-status ${foregroundClass(pairs.textSecondaryOnSurfaceBaseText)}`}
          role="status"
        >
          <span
            className={`source-status__dot ${roleClass("severity.informational")} ${shapeClass("severity.informational")}`}
            aria-hidden="true"
          />
          <span>{preview.sourceLabel}</span>
        </div>
        <label
          className={`theme-control ${foregroundClass(pairs.textSecondaryOnSurfaceBaseText)}`}
        >
          <span>{strings.themeLabel}</span>
          <select
            className={`${pairClass(pairs.textPrimaryOnSurfaceBaseText)} ${borderClass(pairs.borderDefaultOnSurfaceBaseUi)} ${outlineClass(pairs.focusRingOnSurfaceBaseUi)}`}
            value={theme}
            onChange={(event) =>
              onThemeChange(event.currentTarget.value as ThemeChoice)
            }
          >
            {Object.entries(strings.themeOptions).map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </select>
        </label>
      </header>

      <section
        className={`preview-banner ${pairClass(pairs.textPrimaryOnSurfaceOverlayText)}`}
        aria-labelledby="preview-heading"
      >
        <strong id="preview-heading">{strings.previewNotice}</strong>
        <span>{strings.previewExplanation}</span>
      </section>

      <main className="workspace">
        <DeviceRail
          devices={preview.devices}
          selectedDeviceId={selectedDevice.id}
          strings={strings}
          onSelect={setSelectedDeviceId}
        />
        <div className="workspace__center">
          <TopologyMap
            device={selectedDevice}
            selectedNodeId={selectedNode.id}
            strings={strings}
            onSelect={(nodeId) =>
              setSelectedNodeByDevice((current) => ({
                ...current,
                [selectedDevice.id]: nodeId,
              }))
            }
          />
        </div>
        <Inspector
          device={selectedDevice}
          node={selectedNode}
          strings={strings}
        />
      </main>

      <PlanDrawer
        open={planOpen}
        plan={preview.plan}
        strings={strings}
        onToggle={() => setPlanOpen((current) => !current)}
      />
    </div>
  );
}
