#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CaptureStage {
    Before,
    After,
    Final,
}

impl CaptureStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Before => "before",
            Self::After => "after",
            Self::Final => "final",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CaptureTextureClass {
    ColorTarget,
    DepthTarget,
    HistoryTexture,
    ImportedTexture,
}

impl CaptureTextureClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ColorTarget => "color_target",
            Self::DepthTarget => "depth_target",
            Self::HistoryTexture => "history_texture",
            Self::ImportedTexture => "imported_texture",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderPixelCoordinate {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderCapturePointIdentity {
    pub flow_id: String,
    pub pass_id: String,
    pub stage: CaptureStage,
    pub resource_id: String,
    pub texture_class: CaptureTextureClass,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderCaptureIdentity {
    pub frame_index: u64,
    /// Originating prepared render surface; a frame ordinal is not globally unique.
    pub surface_id: u64,
    pub pass_label: String,
    pub capture_point: RenderCapturePointIdentity,
}

impl RenderCaptureIdentity {
    pub const fn surface_id(&self) -> u64 {
        self.surface_id
    }

    pub fn flow_id(&self) -> &str {
        self.capture_point.flow_id.as_str()
    }

    pub fn pass_id(&self) -> &str {
        self.capture_point.pass_id.as_str()
    }

    pub fn stage(&self) -> CaptureStage {
        self.capture_point.stage
    }

    pub fn resource_id(&self) -> &str {
        self.capture_point.resource_id.as_str()
    }

    pub fn texture_class(&self) -> CaptureTextureClass {
        self.capture_point.texture_class
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderCaptureTerminalCode {
    Unmatched,
    Unsupported,
    Disabled,
    Skipped,
    ReadbackFailed,
    ExportFailed,
    Completed,
}

impl RenderCaptureTerminalCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unmatched => "unmatched",
            Self::Unsupported => "unsupported",
            Self::Disabled => "disabled",
            Self::Skipped => "skipped",
            Self::ReadbackFailed => "readback_failed",
            Self::ExportFailed => "export_failed",
            Self::Completed => "completed",
        }
    }

    pub fn is_failure(self) -> bool {
        !matches!(self, Self::Completed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderCaptureTerminalReason {
    pub code: String,
    pub detail: String,
}

impl RenderCaptureTerminalReason {
    pub fn new(code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderCaptureTerminal {
    pub code: RenderCaptureTerminalCode,
    pub reason: Option<RenderCaptureTerminalReason>,
}

impl RenderCaptureTerminal {
    pub fn new(
        code: RenderCaptureTerminalCode,
        reason: Option<RenderCaptureTerminalReason>,
    ) -> Self {
        Self { code, reason }
    }

    pub fn completed() -> Self {
        Self {
            code: RenderCaptureTerminalCode::Completed,
            reason: None,
        }
    }

    pub fn with_reason(
        code: RenderCaptureTerminalCode,
        reason_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            code,
            reason: Some(RenderCaptureTerminalReason::new(reason_code, detail)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderCapturedTexture {
    pub identity: RenderCaptureIdentity,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub bytes_rgba8: Option<Vec<u8>>,
    pub terminal: RenderCaptureTerminal,
}

impl RenderCapturedTexture {
    pub fn sample_pixel_rgba8(&self, point: RenderPixelCoordinate) -> Option<[u8; 4]> {
        let bytes = self.bytes_rgba8.as_ref()?;
        if point.x >= self.width || point.y >= self.height {
            return None;
        }
        let pixel_index = (point.y * self.width + point.x) as usize;
        let byte_index = pixel_index * 4;
        if byte_index + 3 >= bytes.len() {
            return None;
        }
        Some([
            bytes[byte_index],
            bytes[byte_index + 1],
            bytes[byte_index + 2],
            bytes[byte_index + 3],
        ])
    }

    pub fn sample_center_rgba8(&self) -> Option<[u8; 4]> {
        self.sample_pixel_rgba8(RenderPixelCoordinate {
            x: self.width / 2,
            y: self.height / 2,
        })
    }

    pub fn is_completed(&self) -> bool {
        self.terminal.code == RenderCaptureTerminalCode::Completed
    }
}

const MAX_CAPTURE_RECORDS: usize = 64;
const MAX_CAPTURE_BYTES: usize = 64 * 1024 * 1024;

/// Derived, bounded observation view. Not producer-owned GPU state proof.
#[derive(Debug, Clone, runen_ecs::Component, runen_ecs::Resource)]
pub struct RenderCapturedTextureState {
    /// Most recent host handoff; source frames live in each capture identity.
    pub frame_index: u64,
    pub captures: Vec<RenderCapturedTexture>,
    /// Monotonic loss signal: evicted evidence MUST NOT imply verification.
    pub dropped_observations: u64,
    pub last_dropped_identity: Option<RenderCaptureIdentity>,
    max_records: usize,
    max_bytes: usize,
}

impl Default for RenderCapturedTextureState {
    fn default() -> Self {
        Self {
            frame_index: 0,
            captures: Vec::new(),
            dropped_observations: 0,
            last_dropped_identity: None,
            max_records: MAX_CAPTURE_RECORDS,
            max_bytes: MAX_CAPTURE_BYTES,
        }
    }
}

impl RenderCapturedTextureState {
    pub fn observe_frame(&mut self, frame_index: u64, captures: &[RenderCapturedTexture]) {
        self.frame_index = frame_index;
        for capture in captures {
            self.captures.push(capture.clone());
            while self.captures.len() > self.max_records
                || self
                    .captures
                    .iter()
                    .filter_map(|c| c.bytes_rgba8.as_ref())
                    .map(Vec::len)
                    .fold(0usize, usize::saturating_add)
                    > self.max_bytes
            {
                let evicted = self.captures.remove(0);
                self.dropped_observations = self.dropped_observations.saturating_add(1);
                self.last_dropped_identity = Some(evicted.identity);
            }
        }
    }

    /// Exact source lookup; this is observation, not semantic verification.
    pub fn find_from_source(
        &self,
        frame_index: u64,
        surface_id: u64,
        flow_id: &str,
        pass_id: &str,
        stage: CaptureStage,
        resource_id: &str,
    ) -> Option<&RenderCapturedTexture> {
        self.captures.iter().rev().find(|c| {
            let i = &c.identity;
            i.frame_index == frame_index
                && i.surface_id() == surface_id
                && i.flow_id() == flow_id
                && i.pass_id() == pass_id
                && i.stage() == stage
                && i.resource_id() == resource_id
        })
    }

    /// Single-surface convenience; ambiguous matching surfaces fail closed.
    pub fn find(
        &self,
        flow_id: &str,
        pass_id: &str,
        stage: CaptureStage,
        resource_id: &str,
    ) -> Option<&RenderCapturedTexture> {
        let mut latest = None;
        let mut surface = None;
        for capture in self.captures.iter().filter(|c| {
            let i = &c.identity;
            i.flow_id() == flow_id
                && i.pass_id() == pass_id
                && i.stage() == stage
                && i.resource_id() == resource_id
        }) {
            if surface.is_some_and(|id| id != capture.identity.surface_id()) {
                return None;
            }
            surface = Some(capture.identity.surface_id());
            latest = Some(capture);
        }
        latest
    }

    pub fn sample_pixel(
        &self,
        flow_id: &str,
        pass_id: &str,
        stage: CaptureStage,
        resource_id: &str,
        point: RenderPixelCoordinate,
    ) -> Option<[u8; 4]> {
        self.find(flow_id, pass_id, stage, resource_id)?
            .sample_pixel_rgba8(point)
    }

    #[cfg(test)]
    fn with_limits(max_records: usize, max_bytes: usize) -> Self {
        Self {
            max_records,
            max_bytes,
            ..Self::default()
        }
    }
}

pub fn assert_sampled_pixel_eq(
    capture: &RenderCapturedTexture,
    point: RenderPixelCoordinate,
    expected: [u8; 4],
    tolerance: u8,
) {
    let sampled = capture
        .sample_pixel_rgba8(point)
        .unwrap_or_else(|| panic!("missing sampled pixel at ({}, {})", point.x, point.y));
    for (channel, (actual, expected_value)) in sampled.into_iter().zip(expected).enumerate() {
        let distance = actual.abs_diff(expected_value);
        assert!(
            distance <= tolerance,
            "pixel mismatch on channel {} at ({}, {}): actual={} expected={} tolerance={}",
            channel,
            point.x,
            point.y,
            actual,
            expected_value,
            tolerance
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capture_with_test_pixels() -> RenderCapturedTexture {
        RenderCapturedTexture {
            identity: RenderCaptureIdentity {
                frame_index: 1,
                surface_id: 1,
                pass_label: "pass.compose".to_string(),
                capture_point: RenderCapturePointIdentity {
                    flow_id: "flow.main".to_string(),
                    pass_id: "pass.compose".to_string(),
                    stage: CaptureStage::After,
                    resource_id: "surface.color".to_string(),
                    texture_class: CaptureTextureClass::ImportedTexture,
                },
            },
            width: 2,
            height: 2,
            format: "Rgba8Unorm".to_string(),
            bytes_rgba8: Some(vec![
                1, 2, 3, 255, 4, 5, 6, 255, 7, 8, 9, 255, 10, 11, 12, 255,
            ]),
            terminal: RenderCaptureTerminal::completed(),
        }
    }

    #[test]
    fn capture_state_finds_and_samples_pixels() {
        let capture = capture_with_test_pixels();
        let mut state = RenderCapturedTextureState::default();
        state.observe_frame(7, std::slice::from_ref(&capture));

        let sampled = state
            .sample_pixel(
                "flow.main",
                "pass.compose",
                CaptureStage::After,
                "surface.color",
                RenderPixelCoordinate { x: 1, y: 0 },
            )
            .expect("pixel should be available");

        assert_eq!(sampled, [4, 5, 6, 255]);
        assert_eq!(state.frame_index, 7);
    }

    #[test]
    fn captures_from_two_surfaces_do_not_overwrite_or_alias() {
        let first = capture_with_test_pixels();
        let mut second = first.clone();
        second.identity.surface_id = 2;
        second.bytes_rgba8 = Some(vec![99; 16]);
        let mut state = RenderCapturedTextureState::default();
        state.observe_frame(8, &[first]);
        state.observe_frame(8, &[second]);
        assert_eq!(state.captures.len(), 2);
        assert_eq!(state.dropped_observations, 0);
        assert!(
            state
                .find(
                    "flow.main",
                    "pass.compose",
                    CaptureStage::After,
                    "surface.color"
                )
                .is_none()
        );
        assert_eq!(
            state
                .find_from_source(
                    1,
                    1,
                    "flow.main",
                    "pass.compose",
                    CaptureStage::After,
                    "surface.color"
                )
                .unwrap()
                .sample_center_rgba8(),
            Some([10, 11, 12, 255])
        );
        assert_eq!(
            state
                .find_from_source(
                    1,
                    2,
                    "flow.main",
                    "pass.compose",
                    CaptureStage::After,
                    "surface.color"
                )
                .unwrap()
                .sample_center_rgba8(),
            Some([99; 4])
        );
    }

    #[test]
    fn bounded_capture_window_reports_explicit_loss() {
        let mut state = RenderCapturedTextureState::with_limits(2, 32);
        for frame in 1..=3 {
            let mut capture = capture_with_test_pixels();
            capture.identity.frame_index = frame;
            state.observe_frame(frame, &[capture]);
        }
        assert_eq!(state.captures.len(), 2);
        assert_eq!(state.dropped_observations, 1);
        assert_eq!(state.last_dropped_identity.as_ref().unwrap().frame_index, 1);
        assert!(
            state
                .find_from_source(
                    1,
                    1,
                    "flow.main",
                    "pass.compose",
                    CaptureStage::After,
                    "surface.color"
                )
                .is_none()
        );
    }

    #[test]
    fn sampled_pixel_assertion_allows_tolerance() {
        let capture = capture_with_test_pixels();
        assert_sampled_pixel_eq(
            &capture,
            RenderPixelCoordinate { x: 0, y: 1 },
            [8, 9, 10, 255],
            1,
        );
    }
}
