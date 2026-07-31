//! Renderer-neutral synthetic shell state and its narrow Slint binding.
//!
//! The fixture preserves the comparison baseline without probing the host.
//! Exact byte counts remain `u64` until presentation, selection keys are
//! independent of paths and labels, and every callback is checked against the
//! currently selectable set before state changes.

use std::{cell::RefCell, collections::BTreeMap, fmt, rc::Rc};

use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::byte_format::format_bytes;
use crate::catalogue::{EnglishCatalogue, ExactByteFactId, Message, TextId, UnknownTokenLabelId};
use crate::generated_ui::{
    DeviceRow, FactRow, PartmanApp, PartmanTextContrastPairId, PartmanUiContrastPairId, TopologyRow,
};
use crate::selection::{
    SelectionId, SelectionIdError, SelectionLookupError, SelectionRegistry, SelectionRegistryError,
};

/// A static fixture or catalogue invariant prevented construction of the shell.
#[derive(Debug)]
pub enum ViewModelError {
    /// A fixture attempted to reserve selection key zero.
    SelectionId(SelectionIdError),
    /// A fixture repeated an opaque selection key in one selectable set.
    SelectionRegistry(SelectionRegistryError),
    /// A generated semantic label was absent from the Rust catalogue.
    Catalogue(UnknownTokenLabelId),
    /// The fixture had no selectable device.
    MissingDevice,
    /// A fixture device had no selectable topology item.
    MissingTopology,
}

impl fmt::Display for ViewModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SelectionId(error) => error.fmt(formatter),
            Self::SelectionRegistry(error) => error.fmt(formatter),
            Self::Catalogue(error) => error.fmt(formatter),
            Self::MissingDevice => EnglishCatalogue::resolve(TextId::MissingDevice).fmt(formatter),
            Self::MissingTopology => {
                EnglishCatalogue::resolve(TextId::MissingTopology).fmt(formatter)
            }
        }
    }
}

impl std::error::Error for ViewModelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SelectionId(error) => Some(error),
            Self::SelectionRegistry(error) => Some(error),
            Self::Catalogue(error) => Some(error),
            Self::MissingDevice | Self::MissingTopology => None,
        }
    }
}

impl From<SelectionIdError> for ViewModelError {
    fn from(error: SelectionIdError) -> Self {
        Self::SelectionId(error)
    }
}

impl From<SelectionRegistryError> for ViewModelError {
    fn from(error: SelectionRegistryError) -> Self {
        Self::SelectionRegistry(error)
    }
}

impl From<UnknownTokenLabelId> for ViewModelError {
    fn from(error: UnknownTokenLabelId) -> Self {
        Self::Catalogue(error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Track {
    Layout,
    Layers,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EntityRole {
    Partition,
    Container,
    Volume,
    Encryption,
    FileSystem,
    Mount,
    FreeSpace,
}

impl EntityRole {
    const fn label_id(self) -> &'static str {
        match self {
            Self::Partition => "meaning.entity.partition",
            Self::Container => "meaning.entity.container",
            Self::Volume => "meaning.entity.volume",
            Self::Encryption => "meaning.entity.encryption",
            Self::FileSystem => "meaning.entity.filesystem",
            Self::Mount => "meaning.entity.mount",
            Self::FreeSpace => "meaning.entity.freeSpace",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Health {
    Healthy,
    Unknown,
}

impl Health {
    const fn text_id(self) -> TextId {
        match self {
            Self::Healthy => TextId::HealthHealthy,
            Self::Unknown => TextId::HealthUnknown,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum InspectorFact {
    Text {
        label: &'static str,
        value: &'static str,
    },
    Bytes {
        label: ExactByteFactId,
        bytes: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TopologyNode {
    id: SelectionId,
    track: Track,
    role: EntityRole,
    label: &'static str,
    subtitle: &'static str,
    size_bytes: u64,
    facts: Vec<InspectorFact>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Device {
    id: SelectionId,
    label: &'static str,
    path: &'static str,
    bus: &'static str,
    size_bytes: u64,
    identity: &'static str,
    health: Health,
    nodes: Vec<TopologyNode>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PreviewPlan {
    title: &'static str,
    summary: &'static str,
    severity_label_id: &'static str,
    steps: &'static [&'static str],
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ShellModel {
    devices: Vec<Device>,
    device_registry: SelectionRegistry,
    topology_registries: BTreeMap<SelectionId, SelectionRegistry>,
    selected_device: SelectionId,
    selected_nodes: BTreeMap<SelectionId, SelectionId>,
    plan: PreviewPlan,
}

impl ShellModel {
    fn synthetic() -> Result<Self, ViewModelError> {
        let devices = synthetic_devices()?;
        let selected_device = devices.first().ok_or(ViewModelError::MissingDevice)?.id;
        let device_registry = SelectionRegistry::new(devices.iter().map(|device| device.id))?;
        let mut topology_registries = BTreeMap::new();
        let mut selected_nodes = BTreeMap::new();
        for device in &devices {
            let first = device
                .nodes
                .first()
                .ok_or(ViewModelError::MissingTopology)?
                .id;
            topology_registries.insert(
                device.id,
                SelectionRegistry::new(device.nodes.iter().map(|node| node.id))?,
            );
            selected_nodes.insert(device.id, first);
        }

        Ok(Self {
            devices,
            device_registry,
            topology_registries,
            selected_device,
            selected_nodes,
            plan: PreviewPlan {
                title: "Resize Windows and reserve dual-boot space",
                summary: "A presentation fixture showing how risk and ordering will appear after the planner exists.",
                severity_label_id: "meaning.severity.dataMoving",
                steps: &[
                    "Validate the current snapshot and identity",
                    "Shrink the NTFS file system",
                    "Shrink the containing partition",
                    "Leave the resulting extent unallocated",
                ],
            },
        })
    }

    fn select_device(&mut self, wire: &str) -> Result<(), SelectionLookupError> {
        let selected = self.device_registry.resolve_wire(wire)?;
        if !self.selected_nodes.contains_key(&selected) {
            return Err(SelectionLookupError::Unknown(selected));
        }
        self.selected_device = selected;
        Ok(())
    }

    fn select_topology(&mut self, wire: &str) -> Result<(), SelectionLookupError> {
        let Some(registry) = self.topology_registries.get(&self.selected_device) else {
            let parsed = SelectionId::from_wire(wire).map_err(SelectionLookupError::InvalidWire)?;
            return Err(SelectionLookupError::Unknown(parsed));
        };
        let selected = registry.resolve_wire(wire)?;
        self.selected_nodes.insert(self.selected_device, selected);
        Ok(())
    }

    fn snapshot(&self) -> Result<ShellSnapshot, ViewModelError> {
        let device = self
            .devices
            .iter()
            .find(|device| device.id == self.selected_device)
            .ok_or(ViewModelError::MissingDevice)?;
        let selected_topology = *self
            .selected_nodes
            .get(&device.id)
            .ok_or(ViewModelError::MissingTopology)?;
        let node = device
            .nodes
            .iter()
            .find(|node| node.id == selected_topology)
            .ok_or(ViewModelError::MissingTopology)?;

        let devices = self
            .devices
            .iter()
            .map(device_row)
            .collect::<Vec<DeviceView>>();
        let layout_count = device
            .nodes
            .iter()
            .filter(|node| node.track == Track::Layout)
            .count();
        let layers_count = device.nodes.len() - layout_count;
        let mut layout_index = 0;
        let mut layers_index = 0;
        let topology = device
            .nodes
            .iter()
            .map(|node| {
                let (track_index, track_count) = match node.track {
                    Track::Layout => {
                        let index = layout_index;
                        layout_index += 1;
                        (index, layout_count)
                    }
                    Track::Layers => {
                        let index = layers_index;
                        layers_index += 1;
                        (index, layers_count)
                    }
                };
                topology_row(node, track_index, track_count)
            })
            .collect::<Result<Vec<TopologyView>, ViewModelError>>()?;

        Ok(ShellSnapshot {
            devices,
            topology,
            inspector: inspector_rows(device, node)?,
            selected_device: self.selected_device.to_wire().to_string(),
            selected_topology: selected_topology.to_wire().to_string(),
            plan_title: self.plan.title.to_owned(),
            plan_summary: self.plan.summary.to_owned(),
            plan_severity: EnglishCatalogue::resolve_token_label(self.plan.severity_label_id)?
                .to_owned(),
            plan_steps: self
                .plan
                .steps
                .iter()
                .map(|step| (*step).to_owned())
                .collect(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DeviceView {
    id: String,
    accessibility_label: String,
    label: String,
    detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TopologyView {
    id: String,
    accessibility_label: String,
    label: String,
    size: String,
    track: Track,
    track_index: usize,
    track_count: usize,
    role: EntityRole,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FactView {
    label: String,
    value: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ShellSnapshot {
    devices: Vec<DeviceView>,
    topology: Vec<TopologyView>,
    inspector: Vec<FactView>,
    selected_device: String,
    selected_topology: String,
    plan_title: String,
    plan_summary: String,
    plan_severity: String,
    plan_steps: Vec<String>,
}

fn device_row(device: &Device) -> DeviceView {
    let size = format_bytes(device.size_bytes);
    DeviceView {
        id: device.id.to_wire().to_string(),
        accessibility_label: format!(
            "{}, {}, {}, {}, {}",
            device.label, size.display, size.exact, device.bus, device.path
        ),
        label: device.label.to_owned(),
        detail: format!("{} · {} · {}", size.display, device.bus, device.path),
    }
}

fn topology_row(
    node: &TopologyNode,
    track_index: usize,
    track_count: usize,
) -> Result<TopologyView, ViewModelError> {
    let size = format_bytes(node.size_bytes);
    let role = EnglishCatalogue::resolve_token_label(node.role.label_id())?;
    Ok(TopologyView {
        id: node.id.to_wire().to_string(),
        accessibility_label: format!(
            "{role}: {}, {}, {}, {}",
            node.label, node.subtitle, size.display, size.exact
        ),
        label: node.label.to_owned(),
        size: size.display,
        track: node.track,
        track_index,
        track_count,
        role: node.role,
    })
}

fn inspector_rows(device: &Device, node: &TopologyNode) -> Result<Vec<FactView>, ViewModelError> {
    let device_size = format_bytes(device.size_bytes);
    let node_size = format_bytes(node.size_bytes);
    let role_label = EnglishCatalogue::resolve_token_label(node.role.label_id())?;
    let mut rows = vec![
        fact(TextId::SelectedLabel, node.label),
        FactView {
            label: role_label.to_owned(),
            value: node.subtitle.to_owned(),
        },
        FactView {
            label: text(TextId::DeviceSizeLabel),
            value: device_size.display,
        },
        FactView {
            label: text(TextId::DeviceExactBytesLabel),
            value: device_size.exact,
        },
        FactView {
            label: text(TextId::SizeLabel),
            value: node_size.display,
        },
        FactView {
            label: text(TextId::ExactBytesLabel),
            value: node_size.exact,
        },
        fact(TextId::PathLabel, device.path),
        fact(TextId::BusLabel, device.bus),
        fact(TextId::IdentityLabel, device.identity),
        fact(
            TextId::HealthLabel,
            EnglishCatalogue::resolve(device.health.text_id()),
        ),
    ];

    for inspector_fact in &node.facts {
        match inspector_fact {
            InspectorFact::Text { label, value } => rows.push(FactView {
                label: (*label).to_owned(),
                value: (*value).to_owned(),
            }),
            InspectorFact::Bytes { label, bytes } => {
                let formatted = format_bytes(*bytes);
                rows.push(FactView {
                    label: text(label.text_id()),
                    value: formatted.display,
                });
                rows.push(FactView {
                    label: EnglishCatalogue::format(Message::ExactFactLabel(*label)).into_owned(),
                    value: formatted.exact,
                });
            }
        }
    }
    Ok(rows)
}

fn fact(label: TextId, value: &str) -> FactView {
    FactView {
        label: text(label),
        value: value.to_owned(),
    }
}

fn text(id: TextId) -> String {
    EnglishCatalogue::resolve(id).to_owned()
}

fn synthetic_devices() -> Result<Vec<Device>, ViewModelError> {
    Ok(vec![synthetic_system_nvme()?, synthetic_external_ssd()?])
}

fn synthetic_system_nvme() -> Result<Device, ViewModelError> {
    Ok(Device {
        id: SelectionId::new(0xD001)?,
        label: "System NVMe",
        path: "fixture://nvme-system",
        bus: "NVMe",
        size_bytes: 1_000_204_886_016,
        identity: "preview:nvme:001",
        health: Health::Healthy,
        nodes: vec![
            node(
                0x1001,
                Track::Layout,
                EntityRole::Partition,
                "EFI",
                "GUID partition · EFI System",
                272_629_760,
                vec![
                    text_fact("Partition type", "EFI System"),
                    byte_fact(ExactByteFactId::StartOffset, 1_048_576),
                ],
            )?,
            node(
                0x1002,
                Track::Layout,
                EntityRole::Partition,
                "Windows",
                "GUID partition · Basic data",
                749_731_708_928,
                vec![
                    text_fact("Partition type", "Microsoft Basic Data"),
                    byte_fact(ExactByteFactId::StartOffset, 273_678_336),
                ],
            )?,
            node(
                0x1003,
                Track::Layout,
                EntityRole::Partition,
                "Recovery",
                "GUID partition · Windows recovery",
                1_073_741_824,
                vec![
                    text_fact("Partition type", "Windows Recovery"),
                    text_fact("Protected", "Yes"),
                ],
            )?,
            node(
                0x1004,
                Track::Layout,
                EntityRole::FreeSpace,
                "Unallocated",
                "Contiguous free extent",
                249_126_805_504,
                vec![
                    byte_fact(ExactByteFactId::Alignment, 1_048_576),
                    text_fact("Availability", "Unallocated"),
                ],
            )?,
            node(
                0x1005,
                Track::Layers,
                EntityRole::Encryption,
                "BitLocker",
                "Encrypted layer · locked state simulated",
                749_731_708_928,
                vec![
                    text_fact("Protection", "Enabled"),
                    text_fact("Unlock source", "Unavailable in preview"),
                ],
            )?,
            node(
                0x1006,
                Track::Layers,
                EntityRole::FileSystem,
                "NTFS",
                "File system layer",
                749_731_708_928,
                vec![
                    text_fact("File system", "NTFS"),
                    byte_fact(ExactByteFactId::ClusterSize, 4_096),
                ],
            )?,
            node(
                0x1007,
                Track::Layers,
                EntityRole::Mount,
                "System mount",
                "Mounted path association",
                749_731_708_928,
                vec![
                    text_fact("Mount", "C:\\"),
                    text_fact("State", "Preview only"),
                ],
            )?,
        ],
    })
}

fn synthetic_external_ssd() -> Result<Device, ViewModelError> {
    Ok(Device {
        id: SelectionId::new(0xD002)?,
        label: "External SSD",
        path: "fixture://usb-external",
        bus: "USB",
        size_bytes: 500_107_862_016,
        identity: "preview:usb:002",
        health: Health::Unknown,
        nodes: vec![
            node(
                0x2001,
                Track::Layout,
                EntityRole::Partition,
                "APFS store",
                "GUID partition · Apple APFS",
                500_106_813_440,
                vec![
                    text_fact("Partition type", "Apple APFS"),
                    byte_fact(ExactByteFactId::StartOffset, 1_048_576),
                ],
            )?,
            node(
                0x2002,
                Track::Layers,
                EntityRole::Container,
                "APFS container",
                "Container layer",
                500_106_813_440,
                vec![
                    text_fact("Container role", "External"),
                    text_fact("Physical stores", "1"),
                ],
            )?,
            node(
                0x2003,
                Track::Layers,
                EntityRole::Volume,
                "Archive",
                "APFS volume",
                420_000_000_000,
                vec![text_fact("Volume role", "Data"), text_fact("Quota", "None")],
            )?,
            node(
                0x2004,
                Track::Layers,
                EntityRole::FileSystem,
                "APFS",
                "File system layer",
                420_000_000_000,
                vec![
                    text_fact("File system", "APFS"),
                    text_fact("Case sensitive", "No"),
                ],
            )?,
        ],
    })
}

fn node(
    id: u64,
    track: Track,
    role: EntityRole,
    label: &'static str,
    subtitle: &'static str,
    size_bytes: u64,
    facts: Vec<InspectorFact>,
) -> Result<TopologyNode, ViewModelError> {
    Ok(TopologyNode {
        id: SelectionId::new(id)?,
        track,
        role,
        label,
        subtitle,
        size_bytes,
        facts,
    })
}

const fn text_fact(label: &'static str, value: &'static str) -> InspectorFact {
    InspectorFact::Text { label, value }
}

const fn byte_fact(label: ExactByteFactId, bytes: u64) -> InspectorFact {
    InspectorFact::Bytes { label, bytes }
}

/// Populate the AOT component and install fail-closed selection callbacks.
///
/// # Errors
///
/// Returns [`ViewModelError`] if the checked-in fixture or catalogue violates
/// an invariant. UI callbacks never propagate errors and leave the last valid
/// selection unchanged when their wire value is malformed, invented, or stale.
pub fn bind(application: &PartmanApp) -> Result<(), ViewModelError> {
    set_static_text(application)?;
    let model = Rc::new(RefCell::new(ShellModel::synthetic()?));
    let snapshot = model.borrow().snapshot()?;
    apply_snapshot(application, &snapshot);

    application.on_select_device({
        let weak = application.as_weak();
        let model = Rc::clone(&model);
        move |wire| {
            let next = model.try_borrow_mut().ok().and_then(|mut model| {
                model
                    .select_device(wire.as_str())
                    .ok()
                    .and_then(|()| model.snapshot().ok())
            });
            if let (Some(application), Some(snapshot)) = (weak.upgrade(), next) {
                apply_snapshot(&application, &snapshot);
            }
        }
    });
    application.on_select_topology({
        let weak = application.as_weak();
        move |wire| {
            let next = model.try_borrow_mut().ok().and_then(|mut model| {
                model
                    .select_topology(wire.as_str())
                    .ok()
                    .and_then(|()| model.snapshot().ok())
            });
            if let (Some(application), Some(snapshot)) = (weak.upgrade(), next) {
                apply_snapshot(&application, &snapshot);
            }
        }
    });
    Ok(())
}

fn set_static_text(application: &PartmanApp) -> Result<(), ViewModelError> {
    macro_rules! set_text {
        ($setter:ident, $id:expr) => {
            application.$setter(SharedString::from(EnglishCatalogue::resolve($id)))
        };
    }

    set_text!(set_window_title, TextId::AppName);
    set_text!(set_app_name, TextId::AppName);
    set_text!(set_product_qualifier, TextId::ProductQualifier);
    set_text!(set_source_label, TextId::SyntheticSourceLabel);
    set_text!(set_preview_notice, TextId::PreviewNotice);
    set_text!(set_preview_explanation, TextId::PreviewExplanation);
    set_text!(set_read_only_label, TextId::ReadOnlyLabel);
    set_text!(set_device_heading, TextId::DeviceRailHeading);
    set_text!(set_topology_heading, TextId::TopologyHeading);
    set_text!(set_inspection_hint, TextId::InspectionHint);
    set_text!(set_layout_track_label, TextId::LayoutTrackLabel);
    set_text!(set_layers_track_label, TextId::LayersTrackLabel);
    set_text!(set_inspector_heading, TextId::InspectorHeading);
    set_text!(set_selected_label, TextId::SelectedLabel);
    set_text!(set_plan_heading, TextId::PlanHeading);
    set_text!(set_plan_preview_label, TextId::PlanPreviewLabel);
    set_text!(set_plan_steps_heading, TextId::PlanStepsHeading);
    set_text!(set_plan_empty_label, TextId::PlanEmptyLabel);
    set_text!(set_open_plan_label, TextId::OpenPlanLabel);
    set_text!(set_close_plan_label, TextId::ClosePlanLabel);
    set_text!(set_theme_label, TextId::ThemeLabel);
    application.set_theme_system_label(token_text("theme.system")?);
    application.set_theme_dark_label(token_text("theme.dark")?);
    application.set_theme_light_label(token_text("theme.light")?);
    application.set_theme_high_contrast_label(token_text("theme.highContrast")?);
    Ok(())
}

fn token_text(id: &str) -> Result<SharedString, ViewModelError> {
    Ok(SharedString::from(EnglishCatalogue::resolve_token_label(
        id,
    )?))
}

fn apply_snapshot(application: &PartmanApp, snapshot: &ShellSnapshot) {
    application.set_devices(ModelRc::new(VecModel::from(
        snapshot
            .devices
            .iter()
            .map(|device| DeviceRow {
                id: device.id.as_str().into(),
                accessibility_id: device.id.as_str().into(),
                accessibility_label: device.accessibility_label.as_str().into(),
                label: device.label.as_str().into(),
                detail: device.detail.as_str().into(),
            })
            .collect::<Vec<_>>(),
    )));
    application.set_topology_items(ModelRc::new(VecModel::from(
        snapshot
            .topology
            .iter()
            .map(|node| {
                let (accent_pair, accent_uses_ui_pair, ui_accent_pair) = role_accent(node.role);
                TopologyRow {
                    id: node.id.as_str().into(),
                    accessibility_id: node.id.as_str().into(),
                    accessibility_label: node.accessibility_label.as_str().into(),
                    label: node.label.as_str().into(),
                    size: node.size.as_str().into(),
                    track: match node.track {
                        Track::Layout => 0,
                        Track::Layers => 1,
                    },
                    track_index: usize_to_i32(node.track_index),
                    track_count: usize_to_i32(node.track_count),
                    accent_pair,
                    accent_uses_ui_pair,
                    ui_accent_pair,
                }
            })
            .collect::<Vec<_>>(),
    )));
    application.set_inspector_facts(ModelRc::new(VecModel::from(
        snapshot
            .inspector
            .iter()
            .map(|fact| FactRow {
                label: fact.label.as_str().into(),
                value: fact.value.as_str().into(),
            })
            .collect::<Vec<_>>(),
    )));
    application.set_plan_steps(ModelRc::new(VecModel::from(
        snapshot
            .plan_steps
            .iter()
            .map(|step| SharedString::from(step.as_str()))
            .collect::<Vec<_>>(),
    )));
    application.set_selected_device_id(snapshot.selected_device.as_str().into());
    application.set_selected_topology_id(snapshot.selected_topology.as_str().into());
    application.set_device_count_label(
        EnglishCatalogue::format(Message::DeviceCount(snapshot.devices.len()))
            .as_ref()
            .into(),
    );
    application.set_topology_item_count_label(
        EnglishCatalogue::format(Message::TopologyItemCount(snapshot.topology.len()))
            .as_ref()
            .into(),
    );
    application.set_plan_title(snapshot.plan_title.as_str().into());
    application.set_plan_summary(snapshot.plan_summary.as_str().into());
    application.set_plan_severity_label(snapshot.plan_severity.as_str().into());
}

fn usize_to_i32(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

fn role_accent(role: EntityRole) -> (PartmanTextContrastPairId, bool, PartmanUiContrastPairId) {
    let fallback_ui = PartmanUiContrastPairId::Ventityz2efreez53paceToVsurfacez2ebase;
    match role {
        EntityRole::Partition => (
            PartmanTextContrastPairId::Ventityz2epartitionToVsurfacez2ebase,
            false,
            fallback_ui,
        ),
        EntityRole::Container => (
            PartmanTextContrastPairId::Ventityz2econtainerToVsurfacez2ebase,
            false,
            fallback_ui,
        ),
        EntityRole::Volume => (
            PartmanTextContrastPairId::Ventityz2evolumeToVsurfacez2ebase,
            false,
            fallback_ui,
        ),
        EntityRole::Encryption => (
            PartmanTextContrastPairId::Ventityz2eencryptionToVsurfacez2ebase,
            false,
            fallback_ui,
        ),
        EntityRole::FileSystem => (
            PartmanTextContrastPairId::Ventityz2efilesystemToVsurfacez2ebase,
            false,
            fallback_ui,
        ),
        EntityRole::Mount => (
            PartmanTextContrastPairId::Ventityz2emountToVsurfacez2ebase,
            false,
            fallback_ui,
        ),
        EntityRole::FreeSpace => (
            PartmanTextContrastPairId::Vtextz2esecondaryToVsurfacez2ebase,
            true,
            fallback_ui,
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::selection::{SelectionLookupError, SelectionWireError};

    use super::{ShellModel, Track};

    // Requirements: UI-002, UI-003, UI-013
    //   The native candidate preserves the bounded comparison fixture, exact
    //   bytes, two topology tracks, and Rust-owned plan text without touching
    //   platform inventory.
    // Work-Package: WP-030
    // Evidence: synthetic_shell_snapshot_is_complete_and_lossless
    #[test]
    fn synthetic_shell_snapshot_is_complete_and_lossless() {
        let model = ShellModel::synthetic().expect("checked fixture is valid");
        let snapshot = model.snapshot().expect("checked fixture can render");
        assert_eq!(snapshot.devices.len(), 2);
        assert_eq!(snapshot.topology.len(), 7);
        assert_eq!(
            snapshot.devices[0].detail,
            "931.5 GiB · NVMe · fixture://nvme-system"
        );
        assert!(
            snapshot.devices[0]
                .accessibility_label
                .contains("1,000,204,886,016 B")
        );
        assert!(
            snapshot
                .topology
                .iter()
                .any(|node| node.track == Track::Layout && node.track_count == 4)
        );
        assert!(
            snapshot
                .topology
                .iter()
                .any(|node| node.track == Track::Layers && node.track_count == 3)
        );
        assert!(snapshot.inspector.iter().any(|fact| {
            fact.label == "Start offset, exact bytes" && fact.value == "1,048,576 B"
        }));
        assert_eq!(snapshot.plan_steps.len(), 4);
        assert!(!snapshot.plan_title.is_empty());
        assert!(!snapshot.plan_summary.is_empty());
        assert_eq!(snapshot.plan_severity, "Data-moving");
    }

    // Requirements: SAFE-003, SAFE-005, UI-008
    //   Opaque callback IDs are independent of fixture identifiers, stale IDs
    //   from another device fail closed, and each device retains its own last
    //   valid topology selection.
    // Work-Package: WP-030
    // Evidence: shell_selection_is_opaque_scoped_and_fail_closed
    #[test]
    fn shell_selection_is_opaque_scoped_and_fail_closed() {
        let mut model = ShellModel::synthetic().expect("checked fixture is valid");
        let initial = model.snapshot().expect("initial snapshot renders");
        assert!(initial.selected_device.starts_with("sid:"));
        assert!(!initial.selected_device.contains("nvme"));

        let second_device = model.devices[1].id.to_wire().to_string();
        model
            .select_device(&second_device)
            .expect("registered device is selectable");
        let second_node = model.devices[1].nodes[1].id.to_wire().to_string();
        model
            .select_topology(&second_node)
            .expect("current-device node is selectable");

        let stale_first_device_node = model.devices[0].nodes[1].id.to_wire().to_string();
        assert!(matches!(
            model.select_topology(&stale_first_device_node),
            Err(SelectionLookupError::Unknown(_))
        ));
        assert_eq!(
            model
                .snapshot()
                .expect("unchanged snapshot renders")
                .selected_topology,
            second_node
        );
        assert_eq!(
            model.select_device("not-a-selection"),
            Err(SelectionLookupError::InvalidWire(
                SelectionWireError::InvalidLength
            ))
        );

        let first_device = model.devices[0].id.to_wire().to_string();
        model
            .select_device(&first_device)
            .expect("first device remains selectable");
        model
            .select_device(&second_device)
            .expect("second device remains selectable");
        assert_eq!(
            model
                .snapshot()
                .expect("retained snapshot renders")
                .selected_topology,
            second_node
        );
    }
}
