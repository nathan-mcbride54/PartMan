import type { MeaningRole } from "@partman/design-tokens";

export type EntityRole = Extract<MeaningRole, `entity.${string}`>;
export type SeverityRole = Extract<MeaningRole, `severity.${string}`>;
export type DeviceHealth = "healthy" | "attention" | "unknown";

export interface FormattedByteSize {
  readonly displaySize: string;
  readonly exactBytes: string;
}

export type InspectorFact =
  | {
      readonly kind: "text";
      readonly label: string;
      readonly value: string;
    }
  | {
      readonly kind: "byteSize";
      readonly label: string;
      readonly bytes: bigint;
    };

export interface TopologyNode {
  readonly id: string;
  readonly track: "layout" | "layers";
  readonly role: EntityRole;
  readonly label: string;
  readonly subtitle: string;
  readonly sizeBytes: bigint;
  readonly fraction: number;
  readonly facts: readonly InspectorFact[];
}

export interface DevicePreview {
  readonly id: string;
  readonly label: string;
  readonly path: string;
  readonly bus: string;
  readonly sizeBytes: bigint;
  readonly identity: string;
  readonly health: DeviceHealth;
  readonly nodes: readonly TopologyNode[];
}

export interface PendingPlanPreview {
  readonly title: string;
  readonly summary: string;
  readonly severity: SeverityRole;
  readonly steps: readonly string[];
}

export interface WorkspacePreview {
  readonly sourceLabel: string;
  readonly devices: readonly DevicePreview[];
  readonly plan: PendingPlanPreview;
}

export interface UiStrings {
  readonly appName: string;
  readonly productQualifier: string;
  readonly previewNotice: string;
  readonly previewExplanation: string;
  readonly deviceRailHeading: string;
  readonly topologyHeading: string;
  readonly topologyLegendLabel: string;
  readonly layoutTrackLabel: string;
  readonly layersTrackLabel: string;
  readonly inspectorHeading: string;
  readonly planHeading: string;
  readonly planEmptyLabel: string;
  readonly planPreviewLabel: string;
  readonly planStepsHeading: string;
  readonly themeLabel: string;
  readonly themeOptions: Readonly<{
    system: string;
    dark: string;
    light: string;
    "high-contrast": string;
  }>;
  readonly sizeLabel: string;
  readonly exactBytesLabel: string;
  readonly deviceSizeLabel: string;
  readonly deviceExactBytesLabel: string;
  readonly pathLabel: string;
  readonly busLabel: string;
  readonly identityLabel: string;
  readonly healthLabel: string;
  readonly healthOptions: Readonly<Record<DeviceHealth, string>>;
  readonly meaningLabels: Readonly<Record<MeaningRole, string>>;
  readonly formatByteSize: (bytes: bigint) => FormattedByteSize;
  readonly exactFactLabel: (label: string) => string;
  readonly readOnlyLabel: string;
  readonly inspectionHint: string;
  readonly selectedLabel: string;
  readonly deviceCountLabel: (count: number) => string;
  readonly topologyItemCountLabel: (count: number) => string;
  readonly openPlanLabel: string;
  readonly closePlanLabel: string;
}

export type ThemeChoice = "system" | "dark" | "light" | "high-contrast";
