// @generated from schemas/design-tokens.json by packages/design-tokens/generate.mjs
// Do not edit by hand. Run `cargo xtask desktop` to verify this file.

export const tokenSetVersion = "1.0.0";
export const sourceSpecVersion = "4.0.0";

export type ThemeName = "dark" | "high-contrast" | "light";
export type ColorRole = "border.default" | "border.strong" | "entity.container" | "entity.device" | "entity.encryption" | "entity.filesystem" | "entity.freeSpace" | "entity.mount" | "entity.partition" | "entity.volume" | "focus.ring" | "progress.awaitingAuthorization" | "progress.complete" | "progress.executing" | "progress.failed" | "progress.planning" | "progress.rebootPending" | "progress.recovering" | "progress.verifying" | "severity.dataMoving" | "severity.destructive" | "severity.disruptive" | "severity.informational" | "severity.reversible" | "surface.base" | "surface.overlay" | "surface.raised" | "surface.sunken" | "text.muted" | "text.primary" | "text.secondary";
export type MeaningRole = "entity.container" | "entity.device" | "entity.encryption" | "entity.filesystem" | "entity.freeSpace" | "entity.mount" | "entity.partition" | "entity.volume" | "progress.awaitingAuthorization" | "progress.complete" | "progress.executing" | "progress.failed" | "progress.planning" | "progress.rebootPending" | "progress.recovering" | "progress.verifying" | "severity.dataMoving" | "severity.destructive" | "severity.disruptive" | "severity.informational" | "severity.reversible";
export type ContrastKind = "text" | "ui";
export type ContrastPair =
  | readonly ["text.primary", "surface.base", "text"]
  | readonly ["text.primary", "surface.raised", "text"]
  | readonly ["text.primary", "surface.overlay", "text"]
  | readonly ["text.primary", "surface.sunken", "text"]
  | readonly ["text.secondary", "surface.base", "text"]
  | readonly ["text.secondary", "surface.raised", "text"]
  | readonly ["text.muted", "surface.base", "text"]
  | readonly ["text.muted", "surface.raised", "text"]
  | readonly ["border.default", "surface.base", "ui"]
  | readonly ["border.strong", "surface.base", "ui"]
  | readonly ["focus.ring", "surface.base", "ui"]
  | readonly ["focus.ring", "surface.raised", "ui"]
  | readonly ["entity.device", "surface.base", "text"]
  | readonly ["entity.partition", "surface.base", "text"]
  | readonly ["entity.container", "surface.base", "text"]
  | readonly ["entity.volume", "surface.base", "text"]
  | readonly ["entity.encryption", "surface.base", "text"]
  | readonly ["entity.filesystem", "surface.base", "text"]
  | readonly ["entity.mount", "surface.base", "text"]
  | readonly ["entity.freeSpace", "surface.base", "ui"]
  | readonly ["severity.informational", "surface.base", "text"]
  | readonly ["severity.reversible", "surface.base", "text"]
  | readonly ["severity.disruptive", "surface.base", "text"]
  | readonly ["severity.dataMoving", "surface.base", "text"]
  | readonly ["severity.destructive", "surface.base", "text"]
  | readonly ["severity.destructive", "surface.raised", "text"]
  | readonly ["progress.planning", "surface.base", "text"]
  | readonly ["progress.awaitingAuthorization", "surface.base", "text"]
  | readonly ["progress.executing", "surface.base", "text"]
  | readonly ["progress.verifying", "surface.base", "text"]
  | readonly ["progress.rebootPending", "surface.base", "text"]
  | readonly ["progress.recovering", "surface.base", "text"]
  | readonly ["progress.failed", "surface.base", "text"]
  | readonly ["progress.complete", "surface.base", "text"];
export type TextContrastPair = Extract<
  ContrastPair,
  readonly [string, string, "text"]
>;
type ContrastPairKey = "text.primary|surface.base|text" | "text.primary|surface.raised|text" | "text.primary|surface.overlay|text" | "text.primary|surface.sunken|text" | "text.secondary|surface.base|text" | "text.secondary|surface.raised|text" | "text.muted|surface.base|text" | "text.muted|surface.raised|text" | "border.default|surface.base|ui" | "border.strong|surface.base|ui" | "focus.ring|surface.base|ui" | "focus.ring|surface.raised|ui" | "entity.device|surface.base|text" | "entity.partition|surface.base|text" | "entity.container|surface.base|text" | "entity.volume|surface.base|text" | "entity.encryption|surface.base|text" | "entity.filesystem|surface.base|text" | "entity.mount|surface.base|text" | "entity.freeSpace|surface.base|ui" | "severity.informational|surface.base|text" | "severity.reversible|surface.base|text" | "severity.disruptive|surface.base|text" | "severity.dataMoving|surface.base|text" | "severity.destructive|surface.base|text" | "severity.destructive|surface.raised|text" | "progress.planning|surface.base|text" | "progress.awaitingAuthorization|surface.base|text" | "progress.executing|surface.base|text" | "progress.verifying|surface.base|text" | "progress.rebootPending|surface.base|text" | "progress.recovering|surface.base|text" | "progress.failed|surface.base|text" | "progress.complete|surface.base|text";

export interface NonColorChannel {
  readonly icon: string;
  readonly label: string;
  readonly shape: string;
}

export const pairs = {
  textPrimaryOnSurfaceBaseText: ["text.primary", "surface.base", "text"] as const,
  textPrimaryOnSurfaceRaisedText: ["text.primary", "surface.raised", "text"] as const,
  textPrimaryOnSurfaceOverlayText: ["text.primary", "surface.overlay", "text"] as const,
  textPrimaryOnSurfaceSunkenText: ["text.primary", "surface.sunken", "text"] as const,
  textSecondaryOnSurfaceBaseText: ["text.secondary", "surface.base", "text"] as const,
  textSecondaryOnSurfaceRaisedText: ["text.secondary", "surface.raised", "text"] as const,
  textMutedOnSurfaceBaseText: ["text.muted", "surface.base", "text"] as const,
  textMutedOnSurfaceRaisedText: ["text.muted", "surface.raised", "text"] as const,
  borderDefaultOnSurfaceBaseUi: ["border.default", "surface.base", "ui"] as const,
  borderStrongOnSurfaceBaseUi: ["border.strong", "surface.base", "ui"] as const,
  focusRingOnSurfaceBaseUi: ["focus.ring", "surface.base", "ui"] as const,
  focusRingOnSurfaceRaisedUi: ["focus.ring", "surface.raised", "ui"] as const,
  entityDeviceOnSurfaceBaseText: ["entity.device", "surface.base", "text"] as const,
  entityPartitionOnSurfaceBaseText: ["entity.partition", "surface.base", "text"] as const,
  entityContainerOnSurfaceBaseText: ["entity.container", "surface.base", "text"] as const,
  entityVolumeOnSurfaceBaseText: ["entity.volume", "surface.base", "text"] as const,
  entityEncryptionOnSurfaceBaseText: ["entity.encryption", "surface.base", "text"] as const,
  entityFilesystemOnSurfaceBaseText: ["entity.filesystem", "surface.base", "text"] as const,
  entityMountOnSurfaceBaseText: ["entity.mount", "surface.base", "text"] as const,
  entityFreeSpaceOnSurfaceBaseUi: ["entity.freeSpace", "surface.base", "ui"] as const,
  severityInformationalOnSurfaceBaseText: ["severity.informational", "surface.base", "text"] as const,
  severityReversibleOnSurfaceBaseText: ["severity.reversible", "surface.base", "text"] as const,
  severityDisruptiveOnSurfaceBaseText: ["severity.disruptive", "surface.base", "text"] as const,
  severityDataMovingOnSurfaceBaseText: ["severity.dataMoving", "surface.base", "text"] as const,
  severityDestructiveOnSurfaceBaseText: ["severity.destructive", "surface.base", "text"] as const,
  severityDestructiveOnSurfaceRaisedText: ["severity.destructive", "surface.raised", "text"] as const,
  progressPlanningOnSurfaceBaseText: ["progress.planning", "surface.base", "text"] as const,
  progressAwaitingAuthorizationOnSurfaceBaseText: ["progress.awaitingAuthorization", "surface.base", "text"] as const,
  progressExecutingOnSurfaceBaseText: ["progress.executing", "surface.base", "text"] as const,
  progressVerifyingOnSurfaceBaseText: ["progress.verifying", "surface.base", "text"] as const,
  progressRebootPendingOnSurfaceBaseText: ["progress.rebootPending", "surface.base", "text"] as const,
  progressRecoveringOnSurfaceBaseText: ["progress.recovering", "surface.base", "text"] as const,
  progressFailedOnSurfaceBaseText: ["progress.failed", "surface.base", "text"] as const,
  progressCompleteOnSurfaceBaseText: ["progress.complete", "surface.base", "text"] as const,
} as const;

const contrastPairClasses: Readonly<Record<ContrastPairKey, string>> = {
  "text.primary|surface.base|text": "pm-pair-text-primary-on-surface-base-text",
  "text.primary|surface.raised|text": "pm-pair-text-primary-on-surface-raised-text",
  "text.primary|surface.overlay|text": "pm-pair-text-primary-on-surface-overlay-text",
  "text.primary|surface.sunken|text": "pm-pair-text-primary-on-surface-sunken-text",
  "text.secondary|surface.base|text": "pm-pair-text-secondary-on-surface-base-text",
  "text.secondary|surface.raised|text": "pm-pair-text-secondary-on-surface-raised-text",
  "text.muted|surface.base|text": "pm-pair-text-muted-on-surface-base-text",
  "text.muted|surface.raised|text": "pm-pair-text-muted-on-surface-raised-text",
  "border.default|surface.base|ui": "pm-pair-border-default-on-surface-base-ui",
  "border.strong|surface.base|ui": "pm-pair-border-strong-on-surface-base-ui",
  "focus.ring|surface.base|ui": "pm-pair-focus-ring-on-surface-base-ui",
  "focus.ring|surface.raised|ui": "pm-pair-focus-ring-on-surface-raised-ui",
  "entity.device|surface.base|text": "pm-pair-entity-device-on-surface-base-text",
  "entity.partition|surface.base|text": "pm-pair-entity-partition-on-surface-base-text",
  "entity.container|surface.base|text": "pm-pair-entity-container-on-surface-base-text",
  "entity.volume|surface.base|text": "pm-pair-entity-volume-on-surface-base-text",
  "entity.encryption|surface.base|text": "pm-pair-entity-encryption-on-surface-base-text",
  "entity.filesystem|surface.base|text": "pm-pair-entity-filesystem-on-surface-base-text",
  "entity.mount|surface.base|text": "pm-pair-entity-mount-on-surface-base-text",
  "entity.freeSpace|surface.base|ui": "pm-pair-entity-free-space-on-surface-base-ui",
  "severity.informational|surface.base|text": "pm-pair-severity-informational-on-surface-base-text",
  "severity.reversible|surface.base|text": "pm-pair-severity-reversible-on-surface-base-text",
  "severity.disruptive|surface.base|text": "pm-pair-severity-disruptive-on-surface-base-text",
  "severity.dataMoving|surface.base|text": "pm-pair-severity-data-moving-on-surface-base-text",
  "severity.destructive|surface.base|text": "pm-pair-severity-destructive-on-surface-base-text",
  "severity.destructive|surface.raised|text": "pm-pair-severity-destructive-on-surface-raised-text",
  "progress.planning|surface.base|text": "pm-pair-progress-planning-on-surface-base-text",
  "progress.awaitingAuthorization|surface.base|text": "pm-pair-progress-awaiting-authorization-on-surface-base-text",
  "progress.executing|surface.base|text": "pm-pair-progress-executing-on-surface-base-text",
  "progress.verifying|surface.base|text": "pm-pair-progress-verifying-on-surface-base-text",
  "progress.rebootPending|surface.base|text": "pm-pair-progress-reboot-pending-on-surface-base-text",
  "progress.recovering|surface.base|text": "pm-pair-progress-recovering-on-surface-base-text",
  "progress.failed|surface.base|text": "pm-pair-progress-failed-on-surface-base-text",
  "progress.complete|surface.base|text": "pm-pair-progress-complete-on-surface-base-text",
};

const colorRoleClasses: Readonly<Record<MeaningRole, string>> = {
  "entity.container": "pm-role-entity-container",
  "entity.device": "pm-role-entity-device",
  "entity.encryption": "pm-role-entity-encryption",
  "entity.filesystem": "pm-role-entity-filesystem",
  "entity.freeSpace": "pm-role-entity-free-space",
  "entity.mount": "pm-role-entity-mount",
  "entity.partition": "pm-role-entity-partition",
  "entity.volume": "pm-role-entity-volume",
  "progress.awaitingAuthorization": "pm-role-progress-awaiting-authorization",
  "progress.complete": "pm-role-progress-complete",
  "progress.executing": "pm-role-progress-executing",
  "progress.failed": "pm-role-progress-failed",
  "progress.planning": "pm-role-progress-planning",
  "progress.rebootPending": "pm-role-progress-reboot-pending",
  "progress.recovering": "pm-role-progress-recovering",
  "progress.verifying": "pm-role-progress-verifying",
  "severity.dataMoving": "pm-role-severity-data-moving",
  "severity.destructive": "pm-role-severity-destructive",
  "severity.disruptive": "pm-role-severity-disruptive",
  "severity.informational": "pm-role-severity-informational",
  "severity.reversible": "pm-role-severity-reversible",
};

const shapeClasses: Readonly<Record<MeaningRole, string>> = {
  "entity.container": "pm-shape-rect-double",
  "entity.device": "pm-shape-rect-square",
  "entity.encryption": "pm-shape-badge-lock",
  "entity.filesystem": "pm-shape-rect-plain",
  "entity.freeSpace": "pm-shape-rect-dashed",
  "entity.mount": "pm-shape-badge-link",
  "entity.partition": "pm-shape-rect-notched",
  "entity.volume": "pm-shape-rect-rounded",
  "progress.awaitingAuthorization": "pm-shape-dot-hollow",
  "progress.complete": "pm-shape-check",
  "progress.executing": "pm-shape-bar-active",
  "progress.failed": "pm-shape-cross",
  "progress.planning": "pm-shape-dot",
  "progress.rebootPending": "pm-shape-dot-hollow",
  "progress.recovering": "pm-shape-bar-warn",
  "progress.verifying": "pm-shape-bar-check",
  "severity.dataMoving": "pm-shape-chevron-double",
  "severity.destructive": "pm-shape-triangle",
  "severity.disruptive": "pm-shape-chevron",
  "severity.informational": "pm-shape-dot",
  "severity.reversible": "pm-shape-dot",
};

export const nonColorChannels: Readonly<Record<MeaningRole, NonColorChannel>> = {
  "entity.container": { icon: "box", label: "Container", shape: "rect-double" },
  "entity.device": { icon: "drive", label: "Device", shape: "rect-square" },
  "entity.encryption": { icon: "lock", label: "Encrypted", shape: "badge-lock" },
  "entity.filesystem": { icon: "file-tree", label: "File system", shape: "rect-plain" },
  "entity.freeSpace": { icon: "dashed", label: "Free space", shape: "rect-dashed" },
  "entity.mount": { icon: "link", label: "Mounted", shape: "badge-link" },
  "entity.partition": { icon: "slice", label: "Partition", shape: "rect-notched" },
  "entity.volume": { icon: "layers", label: "Volume", shape: "rect-rounded" },
  "progress.awaitingAuthorization": { icon: "shield-question", label: "Waiting for authorization", shape: "dot-hollow" },
  "progress.complete": { icon: "circle-check", label: "Complete", shape: "check" },
  "progress.executing": { icon: "play", label: "Executing", shape: "bar-active" },
  "progress.failed": { icon: "circle-cross", label: "Failed", shape: "cross" },
  "progress.planning": { icon: "compass", label: "Planning", shape: "dot" },
  "progress.rebootPending": { icon: "power-cycle", label: "Reboot pending", shape: "dot-hollow" },
  "progress.recovering": { icon: "life-ring", label: "Recovering", shape: "bar-warn" },
  "progress.verifying": { icon: "magnifier-check", label: "Verifying", shape: "bar-check" },
  "severity.dataMoving": { icon: "arrows-shuffle", label: "Data-moving", shape: "chevron-double" },
  "severity.destructive": { icon: "triangle-exclamation", label: "Destructive", shape: "triangle" },
  "severity.disruptive": { icon: "pause-octagon", label: "Disruptive", shape: "chevron" },
  "severity.informational": { icon: "info", label: "Informational", shape: "dot" },
  "severity.reversible": { icon: "undo", label: "Reversible", shape: "dot" },
};

function pairVariantClass(prefix: string, pair: ContrastPair): string {
  const key = pair.join("|") as ContrastPairKey;
  return contrastPairClasses[key].replace("pm-pair-", `pm-${prefix}-`);
}

export function pairClass(pair: TextContrastPair): string {
  const key = pair.join("|") as ContrastPairKey;
  return contrastPairClasses[key];
}

export function foregroundClass(pair: TextContrastPair): string {
  return pairVariantClass("foreground", pair);
}

export function backgroundClass(pair: ContrastPair): string {
  return pairVariantClass("background", pair);
}

export function borderClass(pair: ContrastPair): string {
  return pairVariantClass("border", pair);
}

export function outlineClass(pair: ContrastPair): string {
  return pairVariantClass("outline", pair);
}

export function roleClass(role: MeaningRole): string {
  return colorRoleClasses[role];
}

export function shapeClass(role: MeaningRole): string {
  return shapeClasses[role];
}

export function channelFor(role: MeaningRole): NonColorChannel {
  return nonColorChannels[role];
}
