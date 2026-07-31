import {
  channelFor,
  foregroundClass,
  pairs,
} from "@partman/design-tokens";
import type { MeaningRole } from "@partman/design-tokens";

const glyphByIcon: Readonly<Record<string, string>> = {
  drive: "▰",
  slice: "◫",
  box: "▣",
  layers: "▤",
  lock: "◆",
  "file-tree": "⌘",
  link: "↗",
  dashed: "┄",
  info: "i",
  undo: "↶",
  "pause-octagon": "Ⅱ",
  "arrows-shuffle": "⇄",
  "triangle-exclamation": "!",
};

interface SemanticMarkProps {
  readonly role: MeaningRole;
  readonly label: string;
  readonly compact?: boolean;
}

export function SemanticMark({
  role,
  label,
  compact = false,
}: SemanticMarkProps) {
  const channel = channelFor(role);
  const glyph = glyphByIcon[channel.icon] ?? "•";
  return (
    <span
      className="semantic-mark"
      data-shape={channel.shape}
      title={label}
      aria-hidden={compact ? "true" : undefined}
    >
      <span className="semantic-mark__glyph" aria-hidden="true">
        {glyph}
      </span>
      {!compact && (
        <span
          className={foregroundClass(
            pairs.textSecondaryOnSurfaceBaseText,
          )}
        >
          {label}
        </span>
      )}
    </span>
  );
}
