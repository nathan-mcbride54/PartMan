//! Closed, dependency-free English catalogue for the native desktop shell.
//!
//! PartMan-authored display text is selected through typed IDs or typed
//! messages. Device labels, paths, identifiers, and future planner output are
//! external model data and do not become catalogue entries. Platform-owned
//! native-dialog text is likewise outside this catalogue, as ADR-0009 records.

use std::{borrow::Cow, fmt};

/// Stable identifier for one non-parameterized PartMan-authored string.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TextId {
    /// Product name.
    AppName,
    /// Short product qualifier shown beside the name.
    ProductQualifier,
    /// Prominent warning that the current content is synthetic.
    PreviewNotice,
    /// Explanation of the synthetic preview's zero-device-access boundary.
    PreviewExplanation,
    /// Synthetic preview source label.
    SyntheticSourceLabel,
    /// Device-rail landmark heading.
    DeviceRailHeading,
    /// Empty state for the device rail.
    DeviceRailEmpty,
    /// Topology landmark heading.
    TopologyHeading,
    /// Accessible name for the topology legend.
    TopologyLegendLabel,
    /// Empty state for the topology region.
    TopologyEmpty,
    /// Physical-layout track heading.
    LayoutTrackLabel,
    /// Logical-layers track heading.
    LayersTrackLabel,
    /// Inspector landmark heading.
    InspectorHeading,
    /// Empty state for the inspector.
    InspectorEmpty,
    /// Pending-plan landmark heading.
    PlanHeading,
    /// Empty state explaining why Apply is unavailable.
    PlanEmptyLabel,
    /// Label marking plan content as illustrative.
    PlanPreviewLabel,
    /// Heading for an illustrative plan's ordered steps.
    PlanStepsHeading,
    /// Label for the theme chooser.
    ThemeLabel,
    /// Label for a human-readable IEC size.
    SizeLabel,
    /// Label for an exact byte count.
    ExactBytesLabel,
    /// Start-offset fact label.
    StartOffsetLabel,
    /// Alignment fact label.
    AlignmentLabel,
    /// Cluster-size fact label.
    ClusterSizeLabel,
    /// Accessible device-size label.
    DeviceSizeLabel,
    /// Accessible exact device-byte label.
    DeviceExactBytesLabel,
    /// Path fact label.
    PathLabel,
    /// Bus fact label.
    BusLabel,
    /// Identity fact label.
    IdentityLabel,
    /// Health fact label.
    HealthLabel,
    /// Healthy-state label.
    HealthHealthy,
    /// Attention-required health-state label.
    HealthAttention,
    /// Unknown-health-state label.
    HealthUnknown,
    /// Read-only state label.
    ReadOnlyLabel,
    /// Inspector selection hint.
    InspectionHint,
    /// Visible and accessible selected-state label.
    SelectedLabel,
    /// Plan-drawer expand action.
    OpenPlanLabel,
    /// Plan-drawer collapse action.
    ClosePlanLabel,
    /// Accessible notice accompanying a visually shortened identifier.
    IdentifierTruncatedLabel,
    /// User-facing failure when a required catalogue entry cannot resolve.
    CatalogueFailure,
    /// User-facing failure when the preview has no device.
    MissingDevice,
    /// User-facing failure when a selected device has no topology item.
    MissingTopology,
    /// User-facing failure when an opaque selection is no longer present.
    SelectionUnavailable,
    /// Actionable-error cause heading.
    ErrorCauseLabel,
    /// Actionable-error unchanged-state heading.
    ErrorUnchangedStateLabel,
    /// Actionable-error safe-next-step heading.
    ErrorSafeNextStepLabel,
    /// Actionable-error diagnostic-details heading.
    ErrorDiagnosticDetailsLabel,
}

impl TextId {
    /// Every static catalogue identifier in stable key order.
    pub const ALL: &'static [Self] = &[
        Self::AppName,
        Self::ProductQualifier,
        Self::PreviewNotice,
        Self::PreviewExplanation,
        Self::SyntheticSourceLabel,
        Self::DeviceRailHeading,
        Self::DeviceRailEmpty,
        Self::TopologyHeading,
        Self::TopologyLegendLabel,
        Self::TopologyEmpty,
        Self::LayoutTrackLabel,
        Self::LayersTrackLabel,
        Self::InspectorHeading,
        Self::InspectorEmpty,
        Self::PlanHeading,
        Self::PlanEmptyLabel,
        Self::PlanPreviewLabel,
        Self::PlanStepsHeading,
        Self::ThemeLabel,
        Self::SizeLabel,
        Self::ExactBytesLabel,
        Self::StartOffsetLabel,
        Self::AlignmentLabel,
        Self::ClusterSizeLabel,
        Self::DeviceSizeLabel,
        Self::DeviceExactBytesLabel,
        Self::PathLabel,
        Self::BusLabel,
        Self::IdentityLabel,
        Self::HealthLabel,
        Self::HealthHealthy,
        Self::HealthAttention,
        Self::HealthUnknown,
        Self::ReadOnlyLabel,
        Self::InspectionHint,
        Self::SelectedLabel,
        Self::OpenPlanLabel,
        Self::ClosePlanLabel,
        Self::IdentifierTruncatedLabel,
        Self::CatalogueFailure,
        Self::MissingDevice,
        Self::MissingTopology,
        Self::SelectionUnavailable,
        Self::ErrorCauseLabel,
        Self::ErrorUnchangedStateLabel,
        Self::ErrorSafeNextStepLabel,
        Self::ErrorDiagnosticDetailsLabel,
    ];

    /// Stable localization key for this string.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::AppName => "app.name",
            Self::ProductQualifier => "app.productQualifier",
            Self::PreviewNotice => "preview.notice",
            Self::PreviewExplanation => "preview.explanation",
            Self::SyntheticSourceLabel => "preview.sourceLabel",
            Self::DeviceRailHeading => "workspace.deviceRail.heading",
            Self::DeviceRailEmpty => "workspace.deviceRail.empty",
            Self::TopologyHeading => "workspace.topology.heading",
            Self::TopologyLegendLabel => "workspace.topology.legendLabel",
            Self::TopologyEmpty => "workspace.topology.empty",
            Self::LayoutTrackLabel => "workspace.topology.layoutTrack",
            Self::LayersTrackLabel => "workspace.topology.layersTrack",
            Self::InspectorHeading => "workspace.inspector.heading",
            Self::InspectorEmpty => "workspace.inspector.empty",
            Self::PlanHeading => "workspace.plan.heading",
            Self::PlanEmptyLabel => "workspace.plan.empty",
            Self::PlanPreviewLabel => "workspace.plan.previewLabel",
            Self::PlanStepsHeading => "workspace.plan.stepsHeading",
            Self::ThemeLabel => "settings.theme.label",
            Self::SizeLabel => "fact.size.displayLabel",
            Self::ExactBytesLabel => "fact.size.exactBytesLabel",
            Self::StartOffsetLabel => "fact.startOffset.label",
            Self::AlignmentLabel => "fact.alignment.label",
            Self::ClusterSizeLabel => "fact.clusterSize.label",
            Self::DeviceSizeLabel => "fact.device.displaySizeLabel",
            Self::DeviceExactBytesLabel => "fact.device.exactBytesLabel",
            Self::PathLabel => "fact.path.label",
            Self::BusLabel => "fact.bus.label",
            Self::IdentityLabel => "fact.identity.label",
            Self::HealthLabel => "fact.health.label",
            Self::HealthHealthy => "health.healthy",
            Self::HealthAttention => "health.attention",
            Self::HealthUnknown => "health.unknown",
            Self::ReadOnlyLabel => "state.readOnly",
            Self::InspectionHint => "workspace.inspector.selectionHint",
            Self::SelectedLabel => "state.selected",
            Self::OpenPlanLabel => "workspace.plan.open",
            Self::ClosePlanLabel => "workspace.plan.close",
            Self::IdentifierTruncatedLabel => "identifier.truncatedDescription",
            Self::CatalogueFailure => "error.catalogueUnavailable",
            Self::MissingDevice => "error.previewMissingDevice",
            Self::MissingTopology => "error.previewMissingTopology",
            Self::SelectionUnavailable => "error.selectionUnavailable",
            Self::ErrorCauseLabel => "error.detail.cause",
            Self::ErrorUnchangedStateLabel => "error.detail.unchangedState",
            Self::ErrorSafeNextStepLabel => "error.detail.safeNextStep",
            Self::ErrorDiagnosticDetailsLabel => "error.detail.diagnosticDetails",
        }
    }
}

/// Closed labels that may qualify an exact byte-valued fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExactByteFactId {
    /// Generic display-size fact.
    Size,
    /// Start-offset fact.
    StartOffset,
    /// Alignment-size fact.
    Alignment,
    /// File-system cluster-size fact.
    ClusterSize,
    /// Whole-device display-size fact.
    DeviceSize,
}

impl ExactByteFactId {
    const fn text_id(self) -> TextId {
        match self {
            Self::Size => TextId::SizeLabel,
            Self::StartOffset => TextId::StartOffsetLabel,
            Self::Alignment => TextId::AlignmentLabel,
            Self::ClusterSize => TextId::ClusterSizeLabel,
            Self::DeviceSize => TextId::DeviceSizeLabel,
        }
    }
}

/// A typed catalogue request, including the small set of parameterized messages.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Message {
    /// Resolve one static [`TextId`].
    Text(TextId),
    /// Accessible description for an exact byte-valued fact.
    ExactFactLabel(ExactByteFactId),
    /// Count of synthetic devices in the rail.
    DeviceCount(usize),
    /// Count of topology items in the current device.
    TopologyItemCount(usize),
}

/// The dependency-free English catalogue selected for PartMan v1.
#[derive(Clone, Copy, Debug, Default)]
pub struct EnglishCatalogue;

impl EnglishCatalogue {
    /// Resolve a non-parameterized catalogue string.
    #[must_use]
    pub const fn resolve(id: TextId) -> &'static str {
        match id {
            TextId::AppName => "PartMan",
            TextId::ProductQualifier => "Storage workspace",
            TextId::PreviewNotice => "Synthetic layout preview",
            TextId::PreviewExplanation => {
                "No disk, volume, mount, or operating-system inventory has been queried."
            }
            TextId::SyntheticSourceLabel => "Synthetic preview · zero device access",
            TextId::DeviceRailHeading => "Devices",
            TextId::DeviceRailEmpty => "No synthetic devices are available.",
            TextId::TopologyHeading => "Topology map",
            TextId::TopologyLegendLabel => "Topology entity legend",
            TextId::TopologyEmpty => "No topology items are available for this device.",
            TextId::LayoutTrackLabel => "Physical layout",
            TextId::LayersTrackLabel => "Logical layers",
            TextId::InspectorHeading => "Inspector",
            TextId::InspectorEmpty => "Select an item to inspect its exact values.",
            TextId::PlanHeading => "Pending plan",
            TextId::PlanEmptyLabel => {
                "Apply is intentionally unavailable. Planning and storage execution are outside this foundation increment."
            }
            TextId::PlanPreviewLabel => "Illustrative only",
            TextId::PlanStepsHeading => "Proposed order",
            TextId::ThemeLabel => "Theme",
            TextId::SizeLabel => "Display size",
            TextId::ExactBytesLabel => "Exact bytes",
            TextId::StartOffsetLabel => "Start offset",
            TextId::AlignmentLabel => "Alignment",
            TextId::ClusterSizeLabel => "Cluster size",
            TextId::DeviceSizeLabel => "Device display size",
            TextId::DeviceExactBytesLabel => "Device exact bytes",
            TextId::PathLabel => "Path",
            TextId::BusLabel => "Bus",
            TextId::IdentityLabel => "Identity",
            TextId::HealthLabel => "Health",
            TextId::HealthHealthy => "Healthy",
            TextId::HealthAttention => "Needs attention",
            TextId::HealthUnknown => "Unknown",
            TextId::ReadOnlyLabel => "Read-only",
            TextId::InspectionHint => {
                "Select a physical extent or logical layer to inspect its exact values."
            }
            TextId::SelectedLabel => "Selected",
            TextId::OpenPlanLabel => "Open drawer",
            TextId::ClosePlanLabel => "Collapse drawer",
            TextId::IdentifierTruncatedLabel => {
                "Identifier shortened visually; the full escaped value is available in the inspector."
            }
            TextId::CatalogueFailure => "A required display label is unavailable.",
            TextId::MissingDevice => "The synthetic preview contains no device.",
            TextId::MissingTopology => "The selected synthetic device contains no topology item.",
            TextId::SelectionUnavailable => "The requested item is no longer available.",
            TextId::ErrorCauseLabel => "Cause",
            TextId::ErrorUnchangedStateLabel => "What remains unchanged",
            TextId::ErrorSafeNextStepLabel => "Safe next step",
            TextId::ErrorDiagnosticDetailsLabel => "Diagnostic details",
        }
    }

    /// Resolve a typed message through the English grammar rules.
    #[must_use]
    pub fn format(message: Message) -> Cow<'static, str> {
        match message {
            Message::Text(id) => Cow::Borrowed(Self::resolve(id)),
            Message::ExactFactLabel(label) => {
                Cow::Owned(format!("{}, exact bytes", Self::resolve(label.text_id())))
            }
            Message::DeviceCount(count) => Cow::Owned(format!(
                "{count} synthetic {}",
                if count == 1 { "device" } else { "devices" }
            )),
            Message::TopologyItemCount(count) => Cow::Owned(format!(
                "{count} topology {}",
                if count == 1 { "item" } else { "items" }
            )),
        }
    }

    /// Resolve a canonical design-token label ID.
    ///
    /// # Errors
    ///
    /// Returns [`UnknownTokenLabelId`] rather than displaying a placeholder
    /// when the generated token boundary asks for an ID this catalogue does not
    /// own.
    pub fn resolve_token_label(id: &str) -> Result<&'static str, UnknownTokenLabelId> {
        TOKEN_LABELS
            .iter()
            .find(|entry| entry.id == id)
            .map(|entry| entry.english)
            .ok_or_else(|| UnknownTokenLabelId { id: id.into() })
    }

    /// Iterate every canonical token label ID owned by this catalogue.
    #[must_use]
    pub fn token_label_ids() -> impl ExactSizeIterator<Item = &'static str> {
        TOKEN_LABELS.iter().map(|entry| entry.id)
    }
}

/// A canonical token label ID had no English catalogue entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownTokenLabelId {
    id: Box<str>,
}

impl UnknownTokenLabelId {
    /// The unknown ID exactly as received.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl fmt::Display for UnknownTokenLabelId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unknown externalized token label ID {:?}",
            self.id
        )
    }
}

impl std::error::Error for UnknownTokenLabelId {}

#[derive(Clone, Copy, Debug)]
struct TokenLabelEntry {
    id: &'static str,
    english: &'static str,
}

const TOKEN_LABELS: &[TokenLabelEntry] = &[
    TokenLabelEntry {
        id: "meaning.entity.container",
        english: "Container",
    },
    TokenLabelEntry {
        id: "meaning.entity.device",
        english: "Device",
    },
    TokenLabelEntry {
        id: "meaning.entity.encryption",
        english: "Encrypted",
    },
    TokenLabelEntry {
        id: "meaning.entity.filesystem",
        english: "File system",
    },
    TokenLabelEntry {
        id: "meaning.entity.freeSpace",
        english: "Free space",
    },
    TokenLabelEntry {
        id: "meaning.entity.mount",
        english: "Mounted",
    },
    TokenLabelEntry {
        id: "meaning.entity.partition",
        english: "Partition",
    },
    TokenLabelEntry {
        id: "meaning.entity.volume",
        english: "Volume",
    },
    TokenLabelEntry {
        id: "meaning.progress.awaitingAuthorization",
        english: "Waiting for authorization",
    },
    TokenLabelEntry {
        id: "meaning.progress.complete",
        english: "Complete",
    },
    TokenLabelEntry {
        id: "meaning.progress.executing",
        english: "Executing",
    },
    TokenLabelEntry {
        id: "meaning.progress.failed",
        english: "Failed",
    },
    TokenLabelEntry {
        id: "meaning.progress.planning",
        english: "Planning",
    },
    TokenLabelEntry {
        id: "meaning.progress.rebootPending",
        english: "Reboot pending",
    },
    TokenLabelEntry {
        id: "meaning.progress.recovering",
        english: "Recovering",
    },
    TokenLabelEntry {
        id: "meaning.progress.verifying",
        english: "Verifying",
    },
    TokenLabelEntry {
        id: "meaning.severity.dataMoving",
        english: "Data-moving",
    },
    TokenLabelEntry {
        id: "meaning.severity.destructive",
        english: "Destructive",
    },
    TokenLabelEntry {
        id: "meaning.severity.disruptive",
        english: "Disruptive",
    },
    TokenLabelEntry {
        id: "meaning.severity.informational",
        english: "Informational",
    },
    TokenLabelEntry {
        id: "meaning.severity.reversible",
        english: "Reversible",
    },
    TokenLabelEntry {
        id: "theme.dark",
        english: "Dark",
    },
    TokenLabelEntry {
        id: "theme.highContrast",
        english: "High contrast",
    },
    TokenLabelEntry {
        id: "theme.light",
        english: "Light",
    },
    TokenLabelEntry {
        id: "theme.system",
        english: "System",
    },
];

#[cfg(test)]
mod tests;
