import {
  borderClass,
  foregroundClass,
  pairClass,
  pairs,
  roleClass,
} from "@partman/design-tokens";

import { SemanticMark } from "./icons";
import type {
  DevicePreview,
  InspectorFact,
  TopologyNode,
  UiStrings,
} from "./types";

interface InspectorProps {
  readonly device: DevicePreview;
  readonly node: TopologyNode;
  readonly strings: UiStrings;
}

function Fact({
  label,
  value,
}: {
  readonly label: string;
  readonly value: string;
}) {
  return (
    <div
      className={`fact ${borderClass(pairs.borderDefaultOnSurfaceBaseUi)}`}
    >
      <dt className={foregroundClass(pairs.textMutedOnSurfaceBaseText)}>
        {label}
      </dt>
      <dd className={foregroundClass(pairs.textSecondaryOnSurfaceBaseText)}>
        {value}
      </dd>
    </div>
  );
}

function InspectorFactRows({
  fact,
  strings,
}: {
  readonly fact: InspectorFact;
  readonly strings: UiStrings;
}) {
  if (fact.kind === "text") {
    return <Fact label={fact.label} value={fact.value} />;
  }

  const size = strings.formatByteSize(fact.bytes);
  return (
    <>
      <Fact label={fact.label} value={size.displaySize} />
      <Fact
        label={strings.exactFactLabel(fact.label)}
        value={size.exactBytes}
      />
    </>
  );
}

export function Inspector({ device, node, strings }: InspectorProps) {
  const deviceSize = strings.formatByteSize(device.sizeBytes);
  const nodeSize = strings.formatByteSize(node.sizeBytes);
  return (
    <aside
      className={`inspector panel ${pairClass(pairs.textPrimaryOnSurfaceBaseText)} ${borderClass(pairs.borderDefaultOnSurfaceBaseUi)}`}
      aria-labelledby="inspector-heading"
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
          <h2 id="inspector-heading">{strings.inspectorHeading}</h2>
        </div>
      </div>

      <div className={`inspector__entity ${roleClass(node.role)}`}>
        <SemanticMark
          role={node.role}
          label={strings.meaningLabels[node.role]}
        />
        <div>
          <strong>{node.label}</strong>
          <p
            className={foregroundClass(pairs.textSecondaryOnSurfaceBaseText)}
          >
            {node.subtitle}
          </p>
        </div>
      </div>

      <dl className="facts">
        <Fact
          label={strings.deviceSizeLabel}
          value={deviceSize.displaySize}
        />
        <Fact
          label={strings.deviceExactBytesLabel}
          value={deviceSize.exactBytes}
        />
        <Fact label={strings.sizeLabel} value={nodeSize.displaySize} />
        <Fact
          label={strings.exactBytesLabel}
          value={nodeSize.exactBytes}
        />
        <Fact label={strings.pathLabel} value={device.path} />
        <Fact label={strings.busLabel} value={device.bus} />
        <Fact label={strings.identityLabel} value={device.identity} />
        <Fact
          label={strings.healthLabel}
          value={strings.healthOptions[device.health]}
        />
        {node.facts.map((fact) => (
          <InspectorFactRows
            key={`${fact.kind}:${fact.label}`}
            fact={fact}
            strings={strings}
          />
        ))}
      </dl>
    </aside>
  );
}
