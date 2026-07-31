import {
  borderClass,
  foregroundClass,
  outlineClass,
  pairClass,
  pairs,
  roleClass,
  shapeClass,
} from "@partman/design-tokens";

import { SemanticMark } from "./icons";
import type { DevicePreview, UiStrings } from "./types";

interface TopologyMapProps {
  readonly device: DevicePreview;
  readonly selectedNodeId: string;
  readonly strings: UiStrings;
  readonly onSelect: (nodeId: string) => void;
}

export function topologyWeight(fraction: number): number {
  if (!Number.isFinite(fraction)) {
    return 1;
  }
  return Math.max(1, Math.min(24, Math.round(fraction * 24)));
}

export function TopologyMap({
  device,
  selectedNodeId,
  strings,
  onSelect,
}: TopologyMapProps) {
  const deviceSize = strings.formatByteSize(device.sizeBytes);
  const tracks = [
    {
      id: "layout",
      label: strings.layoutTrackLabel,
      nodes: device.nodes.filter((node) => node.track === "layout"),
    },
    {
      id: "layers",
      label: strings.layersTrackLabel,
      nodes: device.nodes.filter((node) => node.track === "layers"),
    },
  ] as const;
  const legendRoles = [...new Set(device.nodes.map((node) => node.role))];

  return (
    <section
      className={`topology panel ${pairClass(pairs.textPrimaryOnSurfaceBaseText)} ${borderClass(pairs.borderDefaultOnSurfaceBaseUi)}`}
      aria-labelledby="topology-heading"
    >
      <div
        className={`panel__heading topology__heading ${borderClass(pairs.borderDefaultOnSurfaceBaseUi)}`}
      >
        <div>
          <p
            className={`eyebrow ${foregroundClass(pairs.textMutedOnSurfaceBaseText)}`}
          >
            {device.label}
          </p>
          <h2 id="topology-heading">{strings.topologyHeading}</h2>
        </div>
        <div className="topology__capacity">
          <strong>{deviceSize.displaySize}</strong>
          <span
            className={foregroundClass(pairs.textMutedOnSurfaceBaseText)}
          >
            {strings.topologyItemCountLabel(device.nodes.length)}
          </span>
        </div>
      </div>

      <p
        className={`topology__hint ${foregroundClass(pairs.textSecondaryOnSurfaceBaseText)}`}
      >
        {strings.inspectionHint}
      </p>

      <div className="topology-tracks">
        {tracks.map((track) => (
          <section className="topology-track" key={track.id}>
            <h3
              className={foregroundClass(pairs.textMutedOnSurfaceBaseText)}
            >
              {track.label}
            </h3>
            <div className="topology-strip">
              {track.nodes.map((node) => {
                const selected = node.id === selectedNodeId;
                const size = strings.formatByteSize(node.sizeBytes);
                return (
                  <button
                    className={`topology-node ${pairClass(pairs.textPrimaryOnSurfaceBaseText)} ${roleClass(node.role)} ${shapeClass(node.role)} ${outlineClass(pairs.focusRingOnSurfaceBaseUi)}`}
                    data-weight={topologyWeight(node.fraction)}
                    key={node.id}
                    onClick={() => onSelect(node.id)}
                    type="button"
                    aria-pressed={selected}
                  >
                    <span className="topology-node__kind">
                      <SemanticMark
                        role={node.role}
                        label={strings.meaningLabels[node.role]}
                      />
                    </span>
                    <span
                      className={`topology-node__label ${foregroundClass(pairs.textPrimaryOnSurfaceBaseText)}`}
                    >
                      {node.label}
                    </span>
                    {selected && (
                      <span
                        className={`topology-node__selected ${foregroundClass(pairs.textPrimaryOnSurfaceBaseText)}`}
                      >
                        {strings.selectedLabel}
                      </span>
                    )}
                    <span
                      className={`topology-node__size ${foregroundClass(pairs.textSecondaryOnSurfaceBaseText)}`}
                    >
                      {size.displaySize}
                    </span>
                  </button>
                );
              })}
            </div>
          </section>
        ))}
      </div>

      <div
        className={`topology-legend ${foregroundClass(pairs.textSecondaryOnSurfaceBaseText)} ${borderClass(pairs.borderDefaultOnSurfaceBaseUi)}`}
        aria-label={strings.topologyLegendLabel}
      >
        {legendRoles.map((role) => (
          <span className={roleClass(role)} key={role}>
            <SemanticMark
              role={role}
              label={strings.meaningLabels[role]}
            />
          </span>
        ))}
      </div>
    </section>
  );
}
