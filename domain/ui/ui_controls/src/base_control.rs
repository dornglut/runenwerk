//! File: domain/ui/ui_controls/src/base_control.rs
//! Crate: ui_controls
//! Purpose: UI-local Phase 11 base-control contribution and lowering model.

use ui_layout::{
    UiContainerKind, UiContentState, UiItemIdentityRequirement, UiLargeContentBudget, UiLayoutRole,
    UiScrollRequirement, UiSelectionIdentityRequirement, UiSizeConstraintKind,
    UiVirtualizationRequirement,
};
use ui_program::{RouteCapability, RouteSchemaVersion};
use ui_render_data::{UiExpectedPrimitiveCount, UiPrimitiveFamily};
use ui_schema::{UiSchema, UiSchemaShape};

use crate::catalog::{ControlLayoutInspectionExt, ControlRenderInspectionExt};
use crate::{
    ACTION_PROMPT_CONTROL_KIND_ID, BUTTON_CONTROL_KIND_ID, COLOR_PICKER_CONTROL_KIND_ID,
    ControlAccessibilityDescriptionRequirement, ControlAccessibilityDescriptor,
    ControlAccessibilityLabelRequirement, ControlAccessibilityRole, ControlCatalogIndex,
    ControlCatalogMetadata, ControlFocusRequirement, ControlHostIntentProposal,
    ControlInputDescriptor, ControlInputMode, ControlInspectionDescriptor,
    ControlKeyboardActivation, ControlKeyboardRequirement, ControlKindAuthoringSpec, ControlKindId,
    ControlModuleAuthoringBuilder, ControlModuleDescriptor, ControlPackageAuthoringBuilder,
    ControlPackageDescriptor, ControlPackageVersion, ControlRenderDescriptor,
    ControlRenderEvidenceId, ControlRouteCapabilityDecision, ControlSemanticActionRequirement,
    ControlSemanticHint, ControlSemanticState, ControlStateBindingKind,
    ControlStateBindingRequirement, ControlStateBucket, ControlStateBucketRequirement,
    ControlStateDescriptor, ControlStyleRequirement, ControlStyleRole, ControlTargetProfileRef,
    ControlThemeDescriptor, ControlThemeTokenKind, ControlThemeTokenRequirement,
    ControlThemeTokenRole, ControlValidationState, ControlValueRangeMetadata, ControlVisualState,
    ControlVisualStateRequirement, ControlWheelRequirement, INSPECTOR_FIELD_CONTROL_KIND_ID,
    LABEL_CONTROL_KIND_ID, LIST_VIEW_CONTROL_KIND_ID, RUNENWERK_CONTROL_PACKAGE_ID,
    RUNENWERK_CONTROL_TARGET_EDITOR, TABLE_VIEW_CONTROL_KIND_ID, TREE_VIEW_CONTROL_KIND_ID,
};

pub const BASE_CONTROL_TARGET_KIND_IDS: [&str; 8] = [
    LABEL_CONTROL_KIND_ID,
    BUTTON_CONTROL_KIND_ID,
    INSPECTOR_FIELD_CONTROL_KIND_ID,
    COLOR_PICKER_CONTROL_KIND_ID,
    ACTION_PROMPT_CONTROL_KIND_ID,
    LIST_VIEW_CONTROL_KIND_ID,
    TREE_VIEW_CONTROL_KIND_ID,
    TABLE_VIEW_CONTROL_KIND_ID,
];

pub type ControlCatalog = ControlCatalogIndex;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BaseControlsPlugin;

impl BaseControlsPlugin {
    pub const fn new() -> Self {
        Self
    }

    pub fn contribute(&self, controls: &mut UiControls) {
        controls.add(crate::label::control_contribution());
        controls.add(crate::button::control_contribution());
        controls.add(crate::inspector_field::control_contribution());
        controls.add(crate::color_picker::control_contribution());
        controls.add(crate::action_prompt::control_contribution());
        controls.add(crate::list_view::control_contribution());
        controls.add(crate::tree_view::control_contribution());
        controls.add(crate::table_view::control_contribution());
    }

    pub fn extension(&self) -> UiControls {
        let mut controls = UiControls::new();
        self.contribute(&mut controls);
        controls
    }

    pub fn compile(&self) -> CompiledControlPackage {
        ControlCompiler::new().compile(&self.extension())
    }

    pub fn package(&self) -> ControlPackageDescriptor {
        self.compile().package
    }

    pub fn catalog(&self) -> ControlCatalogIndex {
        self.compile().catalog
    }

    pub fn inspection(&self) -> ControlInspection {
        self.compile().inspection
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiControls {
    contributions: Vec<ControlContribution>,
}

impl UiControls {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, contribution: ControlContribution) {
        self.contributions.push(contribution);
    }

    pub fn with_contribution(mut self, contribution: ControlContribution) -> Self {
        self.add(contribution);
        self
    }

    pub fn contributions(&self) -> &[ControlContribution] {
        &self.contributions
    }

    pub fn len(&self) -> usize {
        self.contributions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.contributions.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ControlContribution {
    pub def: ControlDef,
}

impl ControlContribution {
    pub fn new(def: ControlDef) -> Self {
        Self { def }
    }

    pub fn control_kind_id(&self) -> ControlKindId {
        self.def.control_kind_id()
    }

    pub fn kind_suffix(&self) -> &str {
        &self.def.kind_suffix
    }

    pub fn preset(&self) -> ControlPreset {
        self.def.preset
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ControlDef {
    pub kind_suffix: String,
    pub display_name: String,
    pub description: String,
    pub route_capability: RouteCapability,
    pub route_schema_version: RouteSchemaVersion,
    pub target_profile: ControlTargetProfileRef,
    pub preset: ControlPreset,
    pub category: String,
    pub tags: Vec<String>,
    pub field_groups: Vec<ControlFieldGroup>,
    pub theme_groups: Vec<ControlThemeGroup>,
    pub mount_ineligible_reason: String,
}

impl ControlDef {
    pub fn builder(
        kind_suffix: impl Into<String>,
        display_name: impl Into<String>,
        preset: ControlPreset,
        route_capability: RouteCapability,
    ) -> ControlDefBuilder {
        let kind_suffix = kind_suffix.into();
        let display_name = display_name.into();
        ControlDefBuilder {
            def: Self {
                kind_suffix,
                description: format!(
                    "{display_name} reusable control descriptor with schemas, kernels, diagnostics, fixture, story, host-intent route metadata, and explicit non-mount eligibility until story proof is attached."
                ),
                display_name,
                route_capability,
                route_schema_version: RouteSchemaVersion::new(1),
                target_profile: ControlTargetProfileRef::new(RUNENWERK_CONTROL_TARGET_EDITOR),
                preset,
                category: "base-control".to_owned(),
                tags: vec![
                    "base-control".to_owned(),
                    "catalog-visible".to_owned(),
                    "inspection-ready".to_owned(),
                    "non-mountable".to_owned(),
                ],
                field_groups: Vec::new(),
                theme_groups: Vec::new(),
                mount_ineligible_reason:
                    "runtime mount eligibility requires future story, render, and budget evidence"
                        .to_owned(),
            },
        }
    }

    pub fn control_kind_id(&self) -> ControlKindId {
        ControlKindId::new(format!(
            "{RUNENWERK_CONTROL_PACKAGE_ID}.{}",
            self.kind_suffix
        ))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ControlDefBuilder {
    def: ControlDef,
}

impl ControlDefBuilder {
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.def.description = description.into();
        self
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.def.category = category.into();
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.def.tags.push(tag.into());
        self
    }

    pub fn with_target_profile(mut self, target_profile: ControlTargetProfileRef) -> Self {
        self.def.target_profile = target_profile;
        self
    }

    pub fn with_route_schema_version(mut self, version: RouteSchemaVersion) -> Self {
        self.def.route_schema_version = version;
        self
    }

    pub fn with_field_group(mut self, group: ControlFieldGroup) -> Self {
        self.def.field_groups.push(group);
        self
    }

    pub fn with_theme_group(mut self, group: ControlThemeGroup) -> Self {
        self.def.theme_groups.push(group);
        self
    }

    pub fn with_mount_ineligible_reason(mut self, reason: impl Into<String>) -> Self {
        self.def.mount_ineligible_reason = reason.into();
        self
    }

    pub fn build(self) -> ControlDef {
        self.def
    }

    pub fn build_contribution(self) -> ControlContribution {
        ControlContribution::new(self.build())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ControlPreset {
    Label,
    Button,
    InspectorField,
    ColorPicker,
    ActionPrompt,
    ListView,
    TreeView,
    TableView,
}

impl ControlPreset {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Label => "label",
            Self::Button => "button",
            Self::InspectorField => "inspector-field",
            Self::ColorPicker => "color-picker",
            Self::ActionPrompt => "action-prompt",
            Self::ListView => "list-view",
            Self::TreeView => "tree-view",
            Self::TableView => "table-view",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ControlFieldGroupRole {
    Properties,
    State,
    EventPayload,
}

impl ControlFieldGroupRole {
    const fn schema_suffix(self) -> &'static str {
        match self {
            Self::Properties => "properties",
            Self::State => "state",
            Self::EventPayload => "event",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ControlFieldGroup {
    pub role: ControlFieldGroupRole,
    pub fields: Vec<ControlField>,
}

impl ControlFieldGroup {
    pub fn properties(fields: impl IntoIterator<Item = ControlField>) -> Self {
        Self::new(ControlFieldGroupRole::Properties, fields)
    }

    pub fn state(fields: impl IntoIterator<Item = ControlField>) -> Self {
        Self::new(ControlFieldGroupRole::State, fields)
    }

    pub fn event_payload(fields: impl IntoIterator<Item = ControlField>) -> Self {
        Self::new(ControlFieldGroupRole::EventPayload, fields)
    }

    pub fn new(
        role: ControlFieldGroupRole,
        fields: impl IntoIterator<Item = ControlField>,
    ) -> Self {
        Self {
            role,
            fields: fields.into_iter().collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ControlField {
    pub name: String,
    pub shape: UiSchemaShape,
    pub required: bool,
}

impl ControlField {
    pub fn required(name: impl Into<String>, shape: UiSchemaShape) -> Self {
        Self {
            name: name.into(),
            shape,
            required: true,
        }
    }

    pub fn optional(name: impl Into<String>, shape: UiSchemaShape) -> Self {
        Self {
            name: name.into(),
            shape,
            required: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlThemeGroup {
    pub group_id: String,
    pub tokens: Vec<ControlThemeTokenIntent>,
    pub styles: Vec<ControlStyleIntent>,
    pub visual_states: Vec<ControlVisualStateIntent>,
}

impl ControlThemeGroup {
    pub fn base(group_id: impl Into<String>) -> Self {
        Self::new(group_id)
            .with_token(
                "foreground",
                ControlThemeTokenKind::Color,
                ControlThemeTokenRole::Text,
            )
            .with_token(
                "spacing",
                ControlThemeTokenKind::Spacing,
                ControlThemeTokenRole::Base,
            )
            .with_style(ControlStyleRole::Label, "foreground")
            .with_visual_state(ControlVisualState::Normal)
            .with_optional_visual_state(ControlVisualState::Disabled)
    }

    pub fn surface(group_id: impl Into<String>) -> Self {
        Self::base(group_id)
            .with_token(
                "surface",
                ControlThemeTokenKind::Color,
                ControlThemeTokenRole::Surface,
            )
            .with_token(
                "border",
                ControlThemeTokenKind::Border,
                ControlThemeTokenRole::Border,
            )
            .with_token(
                "radius",
                ControlThemeTokenKind::Radius,
                ControlThemeTokenRole::Base,
            )
            .with_style(ControlStyleRole::Border, "border")
    }

    pub fn new(group_id: impl Into<String>) -> Self {
        Self {
            group_id: group_id.into(),
            tokens: Vec::new(),
            styles: Vec::new(),
            visual_states: Vec::new(),
        }
    }

    pub fn with_token(
        mut self,
        token_name: impl Into<String>,
        kind: ControlThemeTokenKind,
        role: ControlThemeTokenRole,
    ) -> Self {
        self.tokens
            .push(ControlThemeTokenIntent::required(token_name, kind, role));
        self
    }

    pub fn with_optional_token(
        mut self,
        token_name: impl Into<String>,
        kind: ControlThemeTokenKind,
        role: ControlThemeTokenRole,
    ) -> Self {
        self.tokens
            .push(ControlThemeTokenIntent::optional(token_name, kind, role));
        self
    }

    pub fn with_style(mut self, role: ControlStyleRole, token_name: impl Into<String>) -> Self {
        self.styles
            .push(ControlStyleIntent::required(role, token_name));
        self
    }

    pub fn with_optional_style(
        mut self,
        role: ControlStyleRole,
        token_name: impl Into<String>,
    ) -> Self {
        self.styles
            .push(ControlStyleIntent::optional(role, token_name));
        self
    }

    pub fn with_visual_state(mut self, state: ControlVisualState) -> Self {
        self.visual_states
            .push(ControlVisualStateIntent::required(state));
        self
    }

    pub fn with_optional_visual_state(mut self, state: ControlVisualState) -> Self {
        self.visual_states
            .push(ControlVisualStateIntent::optional(state));
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlThemeTokenIntent {
    pub token_name: String,
    pub kind: ControlThemeTokenKind,
    pub role: ControlThemeTokenRole,
    pub required: bool,
}

impl ControlThemeTokenIntent {
    pub fn required(
        token_name: impl Into<String>,
        kind: ControlThemeTokenKind,
        role: ControlThemeTokenRole,
    ) -> Self {
        Self {
            token_name: token_name.into(),
            kind,
            role,
            required: true,
        }
    }

    pub fn optional(
        token_name: impl Into<String>,
        kind: ControlThemeTokenKind,
        role: ControlThemeTokenRole,
    ) -> Self {
        Self {
            token_name: token_name.into(),
            kind,
            role,
            required: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlStyleIntent {
    pub role: ControlStyleRole,
    pub token_name: String,
    pub required: bool,
}

impl ControlStyleIntent {
    pub fn required(role: ControlStyleRole, token_name: impl Into<String>) -> Self {
        Self {
            role,
            token_name: token_name.into(),
            required: true,
        }
    }

    pub fn optional(role: ControlStyleRole, token_name: impl Into<String>) -> Self {
        Self {
            role,
            token_name: token_name.into(),
            required: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlVisualStateIntent {
    pub state: ControlVisualState,
    pub required: bool,
}

impl ControlVisualStateIntent {
    pub const fn required(state: ControlVisualState) -> Self {
        Self {
            state,
            required: true,
        }
    }

    pub const fn optional(state: ControlVisualState) -> Self {
        Self {
            state,
            required: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompiledControlPackage {
    pub package: ControlPackageDescriptor,
    pub controls: Vec<CompiledControl>,
    pub catalog: ControlCatalog,
    pub inspection: ControlInspection,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompiledControl {
    pub contribution: ControlContribution,
    pub module: ControlModuleDescriptor,
    pub layout: crate::ControlLayoutDescriptor,
    pub render: ControlRenderDescriptor,
    pub input: ControlInputDescriptor,
    pub state: ControlStateDescriptor,
    pub theme: ControlThemeDescriptor,
    pub accessibility: ControlAccessibilityDescriptor,
    pub inspection: ControlInspectionDescriptor,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ControlInspection {
    pub controls: Vec<ControlInspectionDescriptor>,
}

impl ControlInspection {
    pub fn descriptor(&self, control_kind_id: &str) -> Option<&ControlInspectionDescriptor> {
        self.controls
            .iter()
            .find(|descriptor| descriptor.control_kind_id == control_kind_id)
    }

    pub fn len(&self) -> usize {
        self.controls.len()
    }

    pub fn is_empty(&self) -> bool {
        self.controls.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ControlCompiler;

impl ControlCompiler {
    pub const fn new() -> Self {
        Self
    }

    pub fn compile(&self, controls: &UiControls) -> CompiledControlPackage {
        let lowered = controls
            .contributions()
            .iter()
            .map(|contribution| self.lower_contribution(contribution))
            .collect::<Vec<_>>();

        let mut package_builder =
            ControlPackageAuthoringBuilder::new(RUNENWERK_CONTROL_PACKAGE_ID, ControlPackageVersion::new(1))
                .with_display_name("Runenwerk base UI controls")
                .with_description("Reusable descriptor package for Runenwerk base controls. Runtime mount eligibility remains disabled until story, render, and budget evidence are attached.")
                .with_category("base-controls")
                .with_tag("control-package")
                .with_target_profile(ControlTargetProfileRef::new(RUNENWERK_CONTROL_TARGET_EDITOR))
                .with_catalog_metadata(ControlCatalogMetadata::new(RUNENWERK_CONTROL_PACKAGE_ID, "Base Controls"));

        for control in &lowered {
            package_builder = package_builder.with_module(control.module.clone());
        }

        let package = package_builder.build();
        let catalog = ControlCatalogIndex::from_packages([&package]);
        let controls = lowered
            .into_iter()
            .map(|control| {
                let inspection = self.lower_inspection(&package, &control);
                CompiledControl {
                    contribution: control.contribution,
                    module: control.module,
                    layout: control.layout,
                    render: control.render,
                    input: control.input,
                    state: control.state,
                    theme: control.theme,
                    accessibility: control.accessibility,
                    inspection,
                }
            })
            .collect::<Vec<_>>();
        let inspection = ControlInspection {
            controls: controls
                .iter()
                .map(|control| control.inspection.clone())
                .collect(),
        };

        CompiledControlPackage {
            package,
            controls,
            catalog,
            inspection,
        }
    }

    pub fn compile_module(&self, contribution: &ControlContribution) -> ControlModuleDescriptor {
        self.lower_module(&contribution.def)
    }

    fn lower_contribution(&self, contribution: &ControlContribution) -> LoweredControl {
        let module = self.lower_module(&contribution.def);
        let kind_id = module.kind.control_kind_id.clone();
        LoweredControl {
            contribution: contribution.clone(),
            module,
            layout: self.lower_layout(&contribution.def, kind_id.clone()),
            render: self.lower_render(&contribution.def, kind_id.clone()),
            input: self.lower_input(&contribution.def, kind_id.clone()),
            state: self.lower_state(&contribution.def, kind_id.clone()),
            theme: self.lower_theme(&contribution.def, kind_id.clone()),
            accessibility: self.lower_accessibility(&contribution.def, kind_id),
        }
    }

    fn lower_module(&self, def: &ControlDef) -> ControlModuleDescriptor {
        let mut spec = ControlKindAuthoringSpec::new(
            RUNENWERK_CONTROL_PACKAGE_ID,
            def.kind_suffix.clone(),
            def.display_name.clone(),
            def.description.clone(),
            def.target_profile.clone(),
            self.lower_schema(def, ControlFieldGroupRole::Properties),
            self.lower_schema(def, ControlFieldGroupRole::State),
            self.lower_schema(def, ControlFieldGroupRole::EventPayload),
            def.route_capability.clone(),
        )
        .with_category(def.category.clone())
        .with_mount_ineligible_reason(def.mount_ineligible_reason.clone());
        spec.route_schema_version = def.route_schema_version;

        for tag in &def.tags {
            spec = spec.with_tag(tag.clone());
        }

        ControlModuleAuthoringBuilder::new(spec).build()
    }

    fn lower_schema(&self, def: &ControlDef, role: ControlFieldGroupRole) -> UiSchema {
        let mut schema = UiSchema::object(
            format!(
                "{RUNENWERK_CONTROL_PACKAGE_ID}.{}.{}",
                def.kind_suffix,
                role.schema_suffix()
            ),
            1,
        );

        for field in def
            .field_groups
            .iter()
            .filter(|group| group.role == role)
            .flat_map(|group| group.fields.iter())
        {
            schema = if field.required {
                schema.with_required_field(field.name.clone(), field.shape.clone())
            } else {
                schema.with_optional_field(field.name.clone(), field.shape.clone())
            };
        }

        schema
    }

    fn lower_layout(
        &self,
        def: &ControlDef,
        kind_id: ControlKindId,
    ) -> crate::ControlLayoutDescriptor {
        match def.preset {
            ControlPreset::Label => add_content_state(
                add_size_constraints(
                    add_layout_roles(
                        crate::ControlLayoutDescriptor::new(kind_id),
                        &[UiLayoutRole::Row],
                    ),
                    &[UiSizeConstraintKind::IntrinsicSize],
                ),
                &[UiContentState::Ready],
            ),
            ControlPreset::Button => base_surface_layout(
                kind_id,
                &[UiLayoutRole::Row, UiLayoutRole::Stack],
                &[
                    UiSizeConstraintKind::MinSize,
                    UiSizeConstraintKind::IntrinsicSize,
                ],
            ),
            ControlPreset::InspectorField => base_surface_layout(
                kind_id,
                &[UiLayoutRole::Row],
                &[
                    UiSizeConstraintKind::FillWidth,
                    UiSizeConstraintKind::IntrinsicSize,
                ],
            ),
            ControlPreset::ColorPicker => base_surface_layout(
                kind_id,
                &[UiLayoutRole::Panel, UiLayoutRole::Stack],
                &[
                    UiSizeConstraintKind::MinSize,
                    UiSizeConstraintKind::PreferredSize,
                ],
            ),
            ControlPreset::ActionPrompt => base_surface_layout(
                kind_id,
                &[UiLayoutRole::Panel, UiLayoutRole::Column],
                &[UiSizeConstraintKind::PreferredSize],
            ),
            ControlPreset::ListView => collection_layout(
                kind_id,
                &[UiLayoutRole::List, UiLayoutRole::VirtualList],
                "list-item-id",
                "selected-list-item-id",
                "list-view-large-content-budget",
                1_000,
                24,
            ),
            ControlPreset::TreeView => collection_layout(
                kind_id,
                &[UiLayoutRole::Tree],
                "tree-node-id",
                "selected-tree-node-id",
                "tree-view-large-content-budget",
                1_000,
                24,
            ),
            ControlPreset::TableView => add_scroll_requirements(
                collection_layout(
                    kind_id,
                    &[UiLayoutRole::Table, UiLayoutRole::VirtualTable],
                    "table-row-id",
                    "selected-table-row-id",
                    "table-view-large-content-budget",
                    10_000,
                    48,
                ),
                &[UiScrollRequirement::AxisX],
            ),
        }
    }

    fn lower_render(&self, def: &ControlDef, kind_id: ControlKindId) -> ControlRenderDescriptor {
        let descriptor = ControlRenderDescriptor::new(kind_id).with_render_evidence(
            ControlRenderEvidenceId::new(format!(
                "{RUNENWERK_CONTROL_PACKAGE_ID}.{}.evidence.render.contract",
                def.kind_suffix
            )),
        );

        match def.preset {
            ControlPreset::Label => add_render_families(
                descriptor,
                &[UiPrimitiveFamily::GlyphRun],
                &[UiExpectedPrimitiveCount::at_least(
                    UiPrimitiveFamily::GlyphRun,
                    1,
                )],
            ),
            ControlPreset::Button | ControlPreset::InspectorField | ControlPreset::ActionPrompt => {
                add_render_families(
                    descriptor,
                    &[
                        UiPrimitiveFamily::Rect,
                        UiPrimitiveFamily::Border,
                        UiPrimitiveFamily::GlyphRun,
                    ],
                    &[
                        UiExpectedPrimitiveCount::at_least(UiPrimitiveFamily::Rect, 1),
                        UiExpectedPrimitiveCount::at_least(UiPrimitiveFamily::GlyphRun, 1),
                    ],
                )
            }
            ControlPreset::ColorPicker => add_render_families(
                descriptor,
                &[
                    UiPrimitiveFamily::Rect,
                    UiPrimitiveFamily::Border,
                    UiPrimitiveFamily::Stroke,
                ],
                &[
                    UiExpectedPrimitiveCount::at_least(UiPrimitiveFamily::Rect, 3),
                    UiExpectedPrimitiveCount::at_least(UiPrimitiveFamily::Stroke, 1),
                ],
            ),
            ControlPreset::ListView | ControlPreset::TreeView | ControlPreset::TableView => {
                add_render_families(
                    descriptor,
                    &[
                        UiPrimitiveFamily::Rect,
                        UiPrimitiveFamily::Clip,
                        UiPrimitiveFamily::GlyphRun,
                    ],
                    &[
                        UiExpectedPrimitiveCount::at_least(UiPrimitiveFamily::Rect, 1),
                        UiExpectedPrimitiveCount::at_least(UiPrimitiveFamily::GlyphRun, 1),
                    ],
                )
            }
        }
    }

    fn lower_input(&self, def: &ControlDef, kind_id: ControlKindId) -> ControlInputDescriptor {
        match def.preset {
            ControlPreset::Label => add_semantic_actions(
                ControlInputDescriptor::new(kind_id).with_modes([ControlInputMode::SemanticAction]),
                &["inspect"],
            ),
            ControlPreset::Button => add_semantic_actions(
                focused_input(
                    kind_id,
                    &[
                        ControlInputMode::Pointer,
                        ControlInputMode::Keyboard,
                        ControlInputMode::SemanticAction,
                        ControlInputMode::TouchReady,
                        ControlInputMode::Controller,
                    ],
                ),
                &["activate"],
            ),
            ControlPreset::InspectorField => add_semantic_actions(
                focused_input(
                    kind_id,
                    &[
                        ControlInputMode::Pointer,
                        ControlInputMode::Keyboard,
                        ControlInputMode::SemanticAction,
                    ],
                ),
                &["commit-value"],
            ),
            ControlPreset::ColorPicker => add_semantic_actions(
                focused_input(
                    kind_id,
                    &[
                        ControlInputMode::Pointer,
                        ControlInputMode::Keyboard,
                        ControlInputMode::SemanticAction,
                        ControlInputMode::TouchReady,
                        ControlInputMode::Controller,
                    ],
                ),
                &["preview-color", "commit-color"],
            ),
            ControlPreset::ActionPrompt => add_semantic_actions(
                focused_input(
                    kind_id,
                    &[
                        ControlInputMode::Pointer,
                        ControlInputMode::Keyboard,
                        ControlInputMode::SemanticAction,
                        ControlInputMode::Controller,
                    ],
                ),
                &["accept", "cancel"],
            ),
            ControlPreset::ListView | ControlPreset::TreeView | ControlPreset::TableView => {
                add_semantic_actions(
                    focused_input(
                        kind_id,
                        &[
                            ControlInputMode::Pointer,
                            ControlInputMode::Wheel,
                            ControlInputMode::Keyboard,
                            ControlInputMode::SemanticAction,
                            ControlInputMode::TouchReady,
                            ControlInputMode::Controller,
                        ],
                    )
                    .with_wheel(ControlWheelRequirement {
                        requires_scroll_delta: true,
                        requires_zoom_delta: false,
                    }),
                    &["select", "navigate"],
                )
            }
        }
    }

    fn lower_state(&self, def: &ControlDef, kind_id: ControlKindId) -> ControlStateDescriptor {
        let route_id = route_id(&def.kind_suffix);
        let descriptor = ControlStateDescriptor::new(kind_id)
            .with_bucket(ControlStateBucketRequirement::new(
                ControlStateBucket::HostFed,
            ))
            .with_bucket(
                ControlStateBucketRequirement::new(ControlStateBucket::Transient).optional(),
            )
            .with_bucket(ControlStateBucketRequirement::new(ControlStateBucket::Focus).optional())
            .with_validation_state(ControlValidationState::Clean)
            .with_validation_state(ControlValidationState::ReadOnly)
            .with_host_intent(
                ControlHostIntentProposal::new(
                    format!(
                        "{RUNENWERK_CONTROL_PACKAGE_ID}.{}.host-intent",
                        def.kind_suffix
                    ),
                    route_id.clone(),
                    def.route_schema_version.value(),
                )
                .with_capability(def.route_capability.as_str()),
            )
            .with_route_decision(ControlRouteCapabilityDecision::not_evaluated(route_id));

        match def.preset {
            ControlPreset::Label => {
                add_bindings(descriptor, &[("label.text", ControlStateBindingKind::Read)])
            }
            ControlPreset::Button => add_bindings(
                descriptor.with_bucket(
                    ControlStateBucketRequirement::new(ControlStateBucket::Preview).optional(),
                ),
                &[("button.activated", ControlStateBindingKind::Option)],
            ),
            ControlPreset::InspectorField => add_bindings(
                descriptor
                    .with_bucket(ControlStateBucketRequirement::new(
                        ControlStateBucket::Preview,
                    ))
                    .with_bucket(ControlStateBucketRequirement::new(
                        ControlStateBucket::Committed,
                    ))
                    .with_validation_state(ControlValidationState::Dirty)
                    .with_validation_state(ControlValidationState::Invalid),
                &[
                    ("inspector-field.value", ControlStateBindingKind::Read),
                    (
                        "inspector-field.proposed-value",
                        ControlStateBindingKind::Write,
                    ),
                ],
            ),
            ControlPreset::ColorPicker => add_bindings(
                descriptor
                    .with_bucket(ControlStateBucketRequirement::new(
                        ControlStateBucket::Preview,
                    ))
                    .with_bucket(ControlStateBucketRequirement::new(
                        ControlStateBucket::Committed,
                    ))
                    .with_validation_state(ControlValidationState::Dirty),
                &[("color-picker.rgba", ControlStateBindingKind::Write)],
            ),
            ControlPreset::ActionPrompt => add_bindings(
                descriptor.with_validation_state(ControlValidationState::PendingValidation),
                &[("action-prompt.answer", ControlStateBindingKind::Option)],
            ),
            ControlPreset::ListView => add_bindings(
                descriptor,
                &[
                    ("list-view.items", ControlStateBindingKind::Collection),
                    ("list-view.selection", ControlStateBindingKind::Selection),
                ],
            ),
            ControlPreset::TreeView => add_bindings(
                descriptor,
                &[
                    ("tree-view.roots", ControlStateBindingKind::Collection),
                    ("tree-view.selection", ControlStateBindingKind::Selection),
                ],
            ),
            ControlPreset::TableView => add_bindings(
                descriptor,
                &[
                    ("table-view.rows", ControlStateBindingKind::Collection),
                    ("table-view.selection", ControlStateBindingKind::Selection),
                ],
            ),
        }
    }

    fn lower_theme(&self, def: &ControlDef, kind_id: ControlKindId) -> ControlThemeDescriptor {
        let mut descriptor = ControlThemeDescriptor::new(kind_id);

        for group in &def.theme_groups {
            for token in &group.tokens {
                let requirement = ControlThemeTokenRequirement::new(
                    theme_token_id(&group.group_id, &token.token_name),
                    token.kind,
                    token.role,
                );
                descriptor = descriptor.with_token(if token.required {
                    requirement
                } else {
                    requirement.optional()
                });
            }

            for style in &group.styles {
                let requirement = ControlStyleRequirement::new(
                    style.role,
                    theme_token_id(&group.group_id, &style.token_name),
                );
                descriptor = descriptor.with_style(if style.required {
                    requirement
                } else {
                    requirement.optional()
                });
            }

            for state in &group.visual_states {
                let requirement = ControlVisualStateRequirement::new(state.state);
                descriptor = descriptor.with_visual_state(if state.required {
                    requirement
                } else {
                    requirement.optional()
                });
            }
        }

        descriptor
    }

    fn lower_accessibility(
        &self,
        def: &ControlDef,
        kind_id: ControlKindId,
    ) -> ControlAccessibilityDescriptor {
        let descriptor = ControlAccessibilityDescriptor::new(kind_id)
            .with_label(ControlAccessibilityLabelRequirement::new(format!(
                "{}.accessible-label",
                def.kind_suffix
            )))
            .with_description(
                ControlAccessibilityDescriptionRequirement::new(format!(
                    "{}.accessible-description",
                    def.kind_suffix
                ))
                .optional(),
            )
            .with_hint(ControlSemanticHint::new(format!(
                "{}.package-summary",
                def.kind_suffix
            )))
            .with_semantic_state(ControlSemanticState::Enabled)
            .with_semantic_state(ControlSemanticState::Disabled);

        match def.preset {
            ControlPreset::Label => add_roles(
                descriptor,
                &[
                    ControlAccessibilityRole::Label,
                    ControlAccessibilityRole::Text,
                ],
            ),
            ControlPreset::Button => add_keyboard(
                focusable(add_roles(descriptor, &[ControlAccessibilityRole::Button]))
                    .with_semantic_state(ControlSemanticState::Pressed),
                &[ControlKeyboardActivation::Activate],
            ),
            ControlPreset::InspectorField => add_keyboard(
                focusable(add_roles(
                    descriptor
                        .with_semantic_state(ControlSemanticState::Readonly)
                        .with_semantic_state(ControlSemanticState::Invalid),
                    &[
                        ControlAccessibilityRole::Panel,
                        ControlAccessibilityRole::Text,
                    ],
                )),
                &[
                    ControlKeyboardActivation::Commit,
                    ControlKeyboardActivation::Cancel,
                ],
            ),
            ControlPreset::ColorPicker => add_keyboard(
                focusable(add_roles(descriptor, &[ControlAccessibilityRole::Custom]))
                    .with_value_range(
                        ControlValueRangeMetadata::new("color-picker.channel-value")
                            .with_minimum()
                            .with_maximum()
                            .with_step(),
                    ),
                &[
                    ControlKeyboardActivation::Commit,
                    ControlKeyboardActivation::Cancel,
                ],
            ),
            ControlPreset::ActionPrompt => add_keyboard(
                add_roles(
                    descriptor,
                    &[
                        ControlAccessibilityRole::Dialog,
                        ControlAccessibilityRole::Panel,
                    ],
                )
                .with_focus(ControlFocusRequirement::focusable().with_focus_return()),
                &[
                    ControlKeyboardActivation::Activate,
                    ControlKeyboardActivation::Cancel,
                ],
            ),
            ControlPreset::ListView => collection_accessibility(
                descriptor,
                &[
                    ControlAccessibilityRole::List,
                    ControlAccessibilityRole::ListItem,
                ],
            ),
            ControlPreset::TreeView => add_keyboard(
                collection_accessibility(
                    descriptor
                        .with_semantic_state(ControlSemanticState::Expanded)
                        .with_semantic_state(ControlSemanticState::Collapsed),
                    &[
                        ControlAccessibilityRole::Tree,
                        ControlAccessibilityRole::TreeItem,
                    ],
                ),
                &[
                    ControlKeyboardActivation::Expand,
                    ControlKeyboardActivation::Collapse,
                ],
            ),
            ControlPreset::TableView => collection_accessibility(
                descriptor,
                &[
                    ControlAccessibilityRole::Table,
                    ControlAccessibilityRole::Row,
                    ControlAccessibilityRole::Cell,
                ],
            ),
        }
    }

    fn lower_inspection(
        &self,
        package: &ControlPackageDescriptor,
        control: &LoweredControl,
    ) -> ControlInspectionDescriptor {
        ControlInspectionDescriptor::from_control_kind(package, &control.module.kind)
            .with_input_summary(&control.input.summary())
            .with_state_summary(&control.state.summary())
            .with_theme_summary(&control.theme.summary())
            .with_accessibility_summary(&control.accessibility.summary())
            .with_control_layout_summary(&control.layout.summary())
            .with_control_render_summary(&control.render.summary())
    }
}

#[derive(Clone, Debug, PartialEq)]
struct LoweredControl {
    contribution: ControlContribution,
    module: ControlModuleDescriptor,
    layout: crate::ControlLayoutDescriptor,
    render: ControlRenderDescriptor,
    input: ControlInputDescriptor,
    state: ControlStateDescriptor,
    theme: ControlThemeDescriptor,
    accessibility: ControlAccessibilityDescriptor,
}

fn base_surface_layout(
    kind_id: ControlKindId,
    roles: &[UiLayoutRole],
    constraints: &[UiSizeConstraintKind],
) -> crate::ControlLayoutDescriptor {
    add_content_state(
        add_size_constraints(
            add_container_kinds(
                add_layout_roles(crate::ControlLayoutDescriptor::new(kind_id), roles),
                &[UiContainerKind::Group],
            ),
            constraints,
        ),
        &[UiContentState::Ready],
    )
}

fn collection_layout(
    kind_id: ControlKindId,
    roles: &[UiLayoutRole],
    item_identity: &str,
    selection_identity: &str,
    budget_id: &str,
    estimated_item_count: u32,
    overscan_budget_items: u32,
) -> crate::ControlLayoutDescriptor {
    add_virtualization_requirements(
        add_content_state(
            add_scroll_requirements(
                add_size_constraints(
                    add_container_kinds(
                        add_layout_roles(crate::ControlLayoutDescriptor::new(kind_id), roles)
                            .with_layout_role(UiLayoutRole::Scroll),
                        &[UiContainerKind::Collection, UiContainerKind::ScrollRegion],
                    ),
                    &[
                        UiSizeConstraintKind::FillWidth,
                        UiSizeConstraintKind::FillHeight,
                    ],
                ),
                &[
                    UiScrollRequirement::Scrollable,
                    UiScrollRequirement::ScrollOwner,
                    UiScrollRequirement::AxisY,
                    UiScrollRequirement::PositionHostOwned,
                ],
            ),
            &[
                UiContentState::Empty,
                UiContentState::Loading,
                UiContentState::Error,
                UiContentState::Overflow,
                UiContentState::Ready,
            ],
        )
        .with_item_identity(UiItemIdentityRequirement::new(item_identity))
        .with_selection_identity(UiSelectionIdentityRequirement::new(selection_identity))
        .with_large_content_budget(
            UiLargeContentBudget::new(budget_id)
                .with_estimated_item_count(estimated_item_count)
                .with_overscan_budget_items(overscan_budget_items),
        ),
        &[
            UiVirtualizationRequirement::Ready,
            UiVirtualizationRequirement::EstimatedItemSize,
            UiVirtualizationRequirement::StableItemIdentity,
            UiVirtualizationRequirement::WindowedRendering,
            UiVirtualizationRequirement::OverscanBudget,
        ],
    )
}

fn focused_input(kind_id: ControlKindId, modes: &[ControlInputMode]) -> ControlInputDescriptor {
    ControlInputDescriptor::new(kind_id)
        .with_modes(modes.iter().copied())
        .with_keyboard(ControlKeyboardRequirement {
            requires_focus: true,
            requires_shortcuts: false,
        })
}

fn collection_accessibility(
    descriptor: ControlAccessibilityDescriptor,
    roles: &[ControlAccessibilityRole],
) -> ControlAccessibilityDescriptor {
    add_keyboard(
        focusable(add_roles(
            descriptor.with_semantic_state(ControlSemanticState::Selected),
            roles,
        )),
        &[
            ControlKeyboardActivation::NavigateNext,
            ControlKeyboardActivation::NavigatePrevious,
        ],
    )
}

fn add_layout_roles(
    mut descriptor: crate::ControlLayoutDescriptor,
    values: &[UiLayoutRole],
) -> crate::ControlLayoutDescriptor {
    for value in values {
        descriptor = descriptor.with_layout_role(*value);
    }
    descriptor
}

fn add_container_kinds(
    mut descriptor: crate::ControlLayoutDescriptor,
    values: &[UiContainerKind],
) -> crate::ControlLayoutDescriptor {
    for value in values {
        descriptor = descriptor.with_container_kind(*value);
    }
    descriptor
}

fn add_size_constraints(
    mut descriptor: crate::ControlLayoutDescriptor,
    values: &[UiSizeConstraintKind],
) -> crate::ControlLayoutDescriptor {
    for value in values {
        descriptor = descriptor.with_size_constraint(*value);
    }
    descriptor
}

fn add_scroll_requirements(
    mut descriptor: crate::ControlLayoutDescriptor,
    values: &[UiScrollRequirement],
) -> crate::ControlLayoutDescriptor {
    for value in values {
        descriptor = descriptor.with_scroll_requirement(*value);
    }
    descriptor
}

fn add_content_state(
    mut descriptor: crate::ControlLayoutDescriptor,
    values: &[UiContentState],
) -> crate::ControlLayoutDescriptor {
    for value in values {
        descriptor = descriptor.with_content_state(*value);
    }
    descriptor
}

fn add_virtualization_requirements(
    mut descriptor: crate::ControlLayoutDescriptor,
    values: &[UiVirtualizationRequirement],
) -> crate::ControlLayoutDescriptor {
    for value in values {
        descriptor = descriptor.with_virtualization_requirement(*value);
    }
    descriptor
}

fn add_render_families(
    mut descriptor: ControlRenderDescriptor,
    families: &[UiPrimitiveFamily],
    counts: &[UiExpectedPrimitiveCount],
) -> ControlRenderDescriptor {
    for family in families {
        descriptor = descriptor.with_required_primitive_family(*family);
    }
    for count in counts {
        descriptor = descriptor.with_expected_primitive_count(count.clone());
    }
    descriptor
}

fn add_semantic_actions(
    mut descriptor: ControlInputDescriptor,
    values: &[&str],
) -> ControlInputDescriptor {
    for value in values {
        descriptor = descriptor.with_semantic_action(ControlSemanticActionRequirement::new(*value));
    }
    descriptor
}

fn add_bindings(
    mut descriptor: ControlStateDescriptor,
    values: &[(&str, ControlStateBindingKind)],
) -> ControlStateDescriptor {
    for (binding_id, kind) in values {
        descriptor =
            descriptor.with_binding(ControlStateBindingRequirement::new(*binding_id, *kind));
    }
    descriptor
}

fn add_roles(
    mut descriptor: ControlAccessibilityDescriptor,
    values: &[ControlAccessibilityRole],
) -> ControlAccessibilityDescriptor {
    for value in values {
        descriptor = descriptor.with_role(*value);
    }
    descriptor
}

fn add_keyboard(
    mut descriptor: ControlAccessibilityDescriptor,
    values: &[ControlKeyboardActivation],
) -> ControlAccessibilityDescriptor {
    for value in values {
        descriptor = descriptor.with_keyboard_activation(*value);
    }
    descriptor
}

fn focusable(descriptor: ControlAccessibilityDescriptor) -> ControlAccessibilityDescriptor {
    descriptor.with_focus(ControlFocusRequirement::focusable())
}

fn theme_token_id(group_id: &str, token: &str) -> String {
    format!("runenwerk.theme.controls.{group_id}.{token}")
}

fn route_id(kind_suffix: &str) -> String {
    format!("{RUNENWERK_CONTROL_PACKAGE_ID}.{kind_suffix}.intent")
}
