use super::*;

#[derive(Debug)]
pub struct PendingCaptureReadback {
    pub selector_index: usize,
    pub identity: RenderCaptureIdentity,
    pub buffer: Buffer,
    pub width: u32,
    pub height: u32,
    pub source_format: TextureFormat,
    pub readback_format: TextureReadbackFormat,
    pub padded_bytes_per_row: u32,
}

#[derive(Debug, Clone)]
struct SelectorRuntimeState {
    selector: RenderCaptureSelector,
    capture_point: Option<RenderCapturePointIdentity>,
    frame_identity: Option<RenderCaptureIdentity>,
    terminal: Option<RenderCaptureTerminal>,
}

impl SelectorRuntimeState {
    fn new(selector: RenderCaptureSelector) -> Self {
        Self {
            selector,
            capture_point: None,
            frame_identity: None,
            terminal: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrameCaptureRuntime {
    pub frame_index: u64,
    selectors: Vec<SelectorRuntimeState>,
}

impl FrameCaptureRuntime {
    pub fn new(
        frame_index: u64,
        debug_control: &RenderDebugControlResource,
        selectors: &[RenderCaptureSelector],
    ) -> Self {
        let mut states = selectors
            .iter()
            .cloned()
            .map(SelectorRuntimeState::new)
            .collect::<Vec<_>>();

        if !debug_control.capture_enabled {
            for state in &mut states {
                state.terminal = Some(RenderCaptureTerminal::with_reason(
                    RenderCaptureTerminalCode::Disabled,
                    "capture_disabled",
                    "capture stage is disabled by RenderDebugControlResource",
                ));
            }
        } else if !debug_control.readback_enabled {
            for state in &mut states {
                state.terminal = Some(RenderCaptureTerminal::with_reason(
                    RenderCaptureTerminalCode::Disabled,
                    "readback_disabled",
                    "readback stage is disabled by RenderDebugControlResource",
                ));
            }
        }

        Self {
            frame_index,
            selectors: states,
        }
    }

    pub fn selectors_len(&self) -> usize {
        self.selectors.len()
    }

    pub fn selector_snapshot(
        &self,
        selector_index: usize,
    ) -> Option<(
        RenderCaptureSelector,
        bool,
        Option<RenderCapturePointIdentity>,
    )> {
        self.selectors.get(selector_index).map(|state| {
            (
                state.selector.clone(),
                state.terminal.is_some(),
                state.capture_point.clone(),
            )
        })
    }

    pub fn should_attempt_stage(&self, stage: CaptureStage) -> bool {
        self.selectors.iter().any(|state| {
            state.selector.stage == stage
                && state.terminal.is_none()
                && state.frame_identity.is_none()
        })
    }

    pub fn set_terminal_with_reason(
        &mut self,
        selector_index: usize,
        code: RenderCaptureTerminalCode,
        reason_code: &str,
        detail: String,
    ) {
        if let Some(state) = self.selectors.get_mut(selector_index)
            && state.terminal.is_none()
        {
            state.terminal = Some(RenderCaptureTerminal::with_reason(
                code,
                reason_code,
                detail,
            ));
        }
    }

    pub fn set_terminal(&mut self, selector_index: usize, terminal: RenderCaptureTerminal) {
        if let Some(state) = self.selectors.get_mut(selector_index) {
            state.terminal = Some(terminal);
        }
    }

    pub fn set_matched_identity(
        &mut self,
        selector_index: usize,
        capture_point: RenderCapturePointIdentity,
        frame_identity: RenderCaptureIdentity,
    ) {
        if let Some(state) = self.selectors.get_mut(selector_index) {
            state.capture_point = Some(capture_point);
            state.frame_identity = Some(frame_identity);
        }
    }

    pub fn finalize_unresolved(&mut self) {
        for state in &mut self.selectors {
            if state.terminal.is_some() {
                continue;
            }
            if state.frame_identity.is_some() {
                state.terminal = Some(RenderCaptureTerminal::with_reason(
                    RenderCaptureTerminalCode::Skipped,
                    "missing_terminal_capture_result",
                    "selector matched a capture point but no terminal capture result was produced",
                ));
                continue;
            }
            state.terminal = Some(RenderCaptureTerminal::with_reason(
                RenderCaptureTerminalCode::Unmatched,
                "selector_unmatched",
                "selector matched no capture point in this frame",
            ));
        }
    }

    pub fn into_plan_and_results(
        self,
    ) -> (ResolvedRenderCapturePlan, Vec<RenderCaptureSelectorResult>) {
        let mut plan = ResolvedRenderCapturePlan {
            frame_index: self.frame_index,
            selectors: Vec::with_capacity(self.selectors.len()),
        };
        let mut results = Vec::<RenderCaptureSelectorResult>::with_capacity(self.selectors.len());

        for (selector_index, state) in self.selectors.into_iter().enumerate() {
            let terminal = state.terminal.unwrap_or_else(|| {
                RenderCaptureTerminal::with_reason(
                    RenderCaptureTerminalCode::Unmatched,
                    "selector_unmatched",
                    "selector matched no capture point in this frame",
                )
            });
            let capture_point = state
                .capture_point
                .clone()
                .unwrap_or_else(|| state.selector.stable_point_fallback());
            let resolution = match terminal.code {
                RenderCaptureTerminalCode::Unmatched => RenderSelectorResolution::Unmatched {
                    reason: terminal.reason.clone().unwrap_or_else(|| {
                        crate::plugins::render::inspect::RenderCaptureTerminalReason::new(
                            "selector_unmatched",
                            "selector matched no capture point in this frame",
                        )
                    }),
                },
                RenderCaptureTerminalCode::Disabled => RenderSelectorResolution::Disabled {
                    reason: terminal.reason.clone().unwrap_or_else(|| {
                        crate::plugins::render::inspect::RenderCaptureTerminalReason::new(
                            "capture_disabled",
                            "capture is disabled",
                        )
                    }),
                },
                RenderCaptureTerminalCode::Unsupported => RenderSelectorResolution::Unsupported {
                    reason: terminal.reason.clone().unwrap_or_else(|| {
                        crate::plugins::render::inspect::RenderCaptureTerminalReason::new(
                            "capture_unsupported",
                            "selector resolved to an unsupported capture path",
                        )
                    }),
                },
                RenderCaptureTerminalCode::Skipped => RenderSelectorResolution::Skipped {
                    reason: terminal.reason.clone().unwrap_or_else(|| {
                        crate::plugins::render::inspect::RenderCaptureTerminalReason::new(
                            "capture_skipped",
                            "capture matched a point but did not produce a completed readback",
                        )
                    }),
                },
                RenderCaptureTerminalCode::ReadbackFailed
                | RenderCaptureTerminalCode::ExportFailed
                | RenderCaptureTerminalCode::Completed => {
                    if let Some(frame_identity) = state.frame_identity.clone() {
                        RenderSelectorResolution::Matched {
                            capture_point: capture_point.clone(),
                            frame_identity,
                        }
                    } else {
                        RenderSelectorResolution::Skipped {
                            reason: terminal.reason.clone().unwrap_or_else(|| {
                                crate::plugins::render::inspect::RenderCaptureTerminalReason::new(
                                    "capture_missing_match",
                                    "selector terminal state did not include a matched frame id",
                                )
                            }),
                        }
                    }
                }
            };

            plan.selectors.push(
                crate::plugins::render::inspect::ResolvedRenderCaptureSelector {
                    selector_index,
                    selector: state.selector.clone(),
                    resolution,
                },
            );
            results.push(RenderCaptureSelectorResult {
                selector_index,
                selector: state.selector,
                capture_point,
                frame_identity: state.frame_identity,
                terminal,
                artifact_path: None,
            });
        }

        (plan, results)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureReadbackMode {
    Rgba8,
    Bgra8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureReadbackFormat {
    pub mode: TextureReadbackMode,
}

#[allow(clippy::too_many_arguments)]
pub fn enqueue_texture_capture_copy(
    device: &Device,
    encoder: &mut CommandEncoder,
    selector_index: usize,
    identity: RenderCaptureIdentity,
    texture: &Texture,
    size: (u32, u32),
    source_format: TextureFormat,
    readback_format: TextureReadbackFormat,
) -> Result<PendingCaptureReadback> {
    let width = size.0.max(1);
    let height = size.1.max(1);
    let unpadded_bytes_per_row = width
        .checked_mul(4)
        .ok_or_else(|| anyhow::anyhow!("capture width overflow for {}", identity.pass_id()))?;
    let padded_bytes_per_row = align_to(unpadded_bytes_per_row, COPY_BYTES_PER_ROW_ALIGNMENT);
    let total_size = (padded_bytes_per_row as u64)
        .checked_mul(height as u64)
        .ok_or_else(|| {
            anyhow::anyhow!("capture buffer size overflow for {}", identity.pass_id())
        })?;

    let buffer = device.create_buffer(&BufferDescriptor {
        label: Some("engine_render_capture_readback"),
        size: total_size,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    encoder.copy_texture_to_buffer(
        TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        TexelCopyBufferInfo {
            buffer: &buffer,
            layout: TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    Ok(PendingCaptureReadback {
        selector_index,
        identity,
        buffer,
        width,
        height,
        source_format,
        readback_format,
        padded_bytes_per_row,
    })
}

pub fn read_capture_back(
    device: &Device,
    pending: PendingCaptureReadback,
) -> (usize, RenderCapturedTexture) {
    let PendingCaptureReadback {
        selector_index,
        identity,
        buffer,
        width,
        height,
        source_format,
        readback_format,
        padded_bytes_per_row,
    } = pending;

    let mut capture = RenderCapturedTexture {
        identity,
        width,
        height,
        format: format!("{:?}", source_format),
        bytes_rgba8: None,
        terminal: RenderCaptureTerminal::completed(),
    };

    let slice = buffer.slice(..);
    let (sender, receiver) = channel();
    slice.map_async(MapMode::Read, move |result| {
        let _ = sender.send(result);
    });

    if let Err(err) = device.poll(PollType::wait_indefinitely()) {
        capture.terminal = RenderCaptureTerminal::with_reason(
            RenderCaptureTerminalCode::ReadbackFailed,
            "device_poll_failed",
            format!("device.poll failed for capture readback: {err}"),
        );
        return (selector_index, capture);
    }

    match receiver.recv() {
        Ok(Ok(())) => {}
        Ok(Err(err)) => {
            capture.terminal = RenderCaptureTerminal::with_reason(
                RenderCaptureTerminalCode::ReadbackFailed,
                "map_async_failed",
                format!("buffer map_async failed: {err}"),
            );
            return (selector_index, capture);
        }
        Err(err) => {
            capture.terminal = RenderCaptureTerminal::with_reason(
                RenderCaptureTerminalCode::ReadbackFailed,
                "map_async_channel_failed",
                format!("buffer map_async channel failed: {err}"),
            );
            return (selector_index, capture);
        }
    }

    let data = slice.get_mapped_range();
    let unpadded_bytes_per_row = width.saturating_mul(4) as usize;
    let expected_padded_len = padded_bytes_per_row as usize * height as usize;
    if data.len() < expected_padded_len {
        capture.terminal = RenderCaptureTerminal::with_reason(
            RenderCaptureTerminalCode::ReadbackFailed,
            "mapped_bytes_too_short",
            format!(
                "mapped capture bytes too short: actual={} expected_at_least={}",
                data.len(),
                expected_padded_len
            ),
        );
        drop(data);
        buffer.unmap();
        return (selector_index, capture);
    }

    let mut rgba = vec![0u8; unpadded_bytes_per_row * height as usize];
    for row in 0..height as usize {
        let source_offset = row * padded_bytes_per_row as usize;
        let source_end = source_offset + unpadded_bytes_per_row;
        let destination_offset = row * unpadded_bytes_per_row;
        let destination_end = destination_offset + unpadded_bytes_per_row;
        let source_row = &data[source_offset..source_end];
        let destination_row = &mut rgba[destination_offset..destination_end];
        match readback_format.mode {
            TextureReadbackMode::Rgba8 => destination_row.copy_from_slice(source_row),
            TextureReadbackMode::Bgra8 => {
                for (pixel_index, chunk) in source_row.chunks_exact(4).enumerate() {
                    let destination_index = pixel_index * 4;
                    destination_row[destination_index] = chunk[2];
                    destination_row[destination_index + 1] = chunk[1];
                    destination_row[destination_index + 2] = chunk[0];
                    destination_row[destination_index + 3] = chunk[3];
                }
            }
        }
    }

    capture.bytes_rgba8 = Some(rgba);
    drop(data);
    buffer.unmap();
    (selector_index, capture)
}

pub fn texture_readback_format(format: TextureFormat) -> Option<TextureReadbackFormat> {
    let mode = match format {
        TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => TextureReadbackMode::Rgba8,
        TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb => TextureReadbackMode::Bgra8,
        _ => return None,
    };
    Some(TextureReadbackFormat { mode })
}

pub fn align_to(value: u32, alignment: u32) -> u32 {
    if alignment == 0 {
        return value;
    }
    let remainder = value % alignment;
    if remainder == 0 {
        value
    } else {
        value + (alignment - remainder)
    }
}
