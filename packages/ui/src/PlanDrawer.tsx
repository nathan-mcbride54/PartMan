import {
  borderClass,
  foregroundClass,
  outlineClass,
  pairClass,
  pairs,
  roleClass,
} from "@partman/design-tokens";

import { SemanticMark } from "./icons";
import type { PendingPlanPreview, UiStrings } from "./types";

interface PlanDrawerProps {
  readonly open: boolean;
  readonly plan: PendingPlanPreview;
  readonly strings: UiStrings;
  readonly onToggle: () => void;
}

export function PlanDrawer({
  open,
  plan,
  strings,
  onToggle,
}: PlanDrawerProps) {
  return (
    <aside
      className={`plan-drawer ${pairClass(pairs.textPrimaryOnSurfaceBaseText)} ${borderClass(pairs.borderStrongOnSurfaceBaseUi)}`}
      aria-labelledby="plan-heading"
      data-open={open}
    >
      <button
        className={`plan-drawer__toggle ${pairClass(pairs.textPrimaryOnSurfaceBaseText)} ${outlineClass(pairs.focusRingOnSurfaceBaseUi)}`}
        aria-expanded={open}
        aria-controls="plan-drawer-content"
        onClick={onToggle}
        type="button"
      >
        <span>
          <span
            className={`eyebrow ${foregroundClass(pairs.textMutedOnSurfaceBaseText)}`}
          >
            {strings.planPreviewLabel}
          </span>
          <strong id="plan-heading">{strings.planHeading}</strong>
        </span>
        <span
          className={foregroundClass(pairs.textSecondaryOnSurfaceBaseText)}
        >
          {open ? strings.closePlanLabel : strings.openPlanLabel}
        </span>
      </button>

      <div
        id="plan-drawer-content"
        className={`plan-drawer__content ${pairClass(pairs.textPrimaryOnSurfaceBaseText)} ${borderClass(pairs.borderDefaultOnSurfaceBaseUi)}`}
        hidden={!open}
      >
        <div className="plan-drawer__summary">
          <div className={`severity-chip ${roleClass(plan.severity)}`}>
            <SemanticMark
              role={plan.severity}
              label={strings.meaningLabels[plan.severity]}
              compact
            />
            <span
              className={foregroundClass(
                pairs.textPrimaryOnSurfaceBaseText,
              )}
            >
              {strings.meaningLabels[plan.severity]}
            </span>
          </div>
          <div>
            <h3>{plan.title}</h3>
            <p
              className={foregroundClass(
                pairs.textSecondaryOnSurfaceBaseText,
              )}
            >
              {plan.summary}
            </p>
          </div>
        </div>
        <div>
          <h3>{strings.planStepsHeading}</h3>
          <ol
            className={`plan-steps ${foregroundClass(pairs.textSecondaryOnSurfaceBaseText)}`}
          >
            {plan.steps.map((step) => (
              <li key={step}>{step}</li>
            ))}
          </ol>
        </div>
        <p
          className={`plan-drawer__empty ${foregroundClass(pairs.textSecondaryOnSurfaceBaseText)} ${borderClass(pairs.borderDefaultOnSurfaceBaseUi)}`}
        >
          {strings.planEmptyLabel}
        </p>
      </div>
    </aside>
  );
}
