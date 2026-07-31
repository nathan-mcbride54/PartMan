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

interface DeviceRailProps {
  readonly devices: readonly DevicePreview[];
  readonly selectedDeviceId: string;
  readonly strings: UiStrings;
  readonly onSelect: (deviceId: string) => void;
}

export function DeviceRail({
  devices,
  selectedDeviceId,
  strings,
  onSelect,
}: DeviceRailProps) {
  return (
    <nav
      className={`device-rail panel ${pairClass(pairs.textPrimaryOnSurfaceBaseText)} ${borderClass(pairs.borderDefaultOnSurfaceBaseUi)}`}
      aria-labelledby="device-rail-heading"
    >
      <div
        className={`panel__heading ${borderClass(pairs.borderDefaultOnSurfaceBaseUi)}`}
      >
        <div>
          <p
            className={`eyebrow ${foregroundClass(pairs.textMutedOnSurfaceBaseText)}`}
          >
            {strings.readOnlyLabel}
          </p>
          <h2 id="device-rail-heading">{strings.deviceRailHeading}</h2>
        </div>
        <span
          className={`count-badge ${foregroundClass(pairs.textSecondaryOnSurfaceBaseText)} ${borderClass(pairs.borderDefaultOnSurfaceBaseUi)}`}
          aria-label={strings.deviceCountLabel(devices.length)}
        >
          {devices.length}
        </span>
      </div>

      <div className="device-list">
        {devices.map((device) => {
          const selected = device.id === selectedDeviceId;
          const size = strings.formatByteSize(device.sizeBytes);
          return (
            <button
              className={`device-card ${pairClass(pairs.textPrimaryOnSurfaceBaseText)} ${roleClass("entity.device")} ${shapeClass("entity.device")} ${outlineClass(pairs.focusRingOnSurfaceBaseUi)}`}
              aria-current={selected ? "true" : undefined}
              key={device.id}
              onClick={() => onSelect(device.id)}
              type="button"
            >
              <span className="device-card__topline">
                <SemanticMark
                  role="entity.device"
                  label={strings.meaningLabels["entity.device"]}
                  compact
                />
                <span
                  className={`device-card__name ${foregroundClass(pairs.textPrimaryOnSurfaceBaseText)}`}
                >
                  {device.label}
                </span>
                {selected && (
                  <span
                    className={`selected-pill ${foregroundClass(pairs.textPrimaryOnSurfaceBaseText)}`}
                  >
                    {strings.selectedLabel}
                  </span>
                )}
              </span>
              <span
                className={`device-card__size ${foregroundClass(pairs.textSecondaryOnSurfaceBaseText)}`}
              >
                {size.displaySize}
              </span>
              <span
                className={`device-card__meta ${foregroundClass(pairs.textMutedOnSurfaceBaseText)}`}
              >
                {device.bus} · {device.path}
              </span>
            </button>
          );
        })}
      </div>
    </nav>
  );
}
