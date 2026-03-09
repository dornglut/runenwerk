impl Renderer {
    // Owner: Engine Renderer - Frame Graph Resolution and Diagnostics
    fn builtin_pass_executor(
        executor: BuiltinRenderPassExecutor,
    ) -> &'static dyn FramePassExecutor {
        match executor {
            BuiltinRenderPassExecutor::Compute => &BUILTIN_COMPUTE_NOOP_PASS_EXECUTOR,
            BuiltinRenderPassExecutor::Compose => &BUILTIN_COMPOSE_NOOP_PASS_EXECUTOR,
            BuiltinRenderPassExecutor::MeshOverlay => &MESH_OVERLAY_NOOP_PASS_EXECUTOR,
            BuiltinRenderPassExecutor::UiComposite => &UI_COMPOSITE_PASS_EXECUTOR,
        }
    }

    fn stable_hash<T: Hash>(value: &T) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    fn log_frame_graph_diagnostics(
        &mut self,
        world_scene: &str,
        overlay_scene: &str,
        registry_revision: u64,
        diagnostics: &FrameGraphCompileDiagnostics,
    ) {
        if !diagnostics.has_issues() {
            if self.last_frame_graph_diagnostics_hash.take().is_some() {
                tracing::info!(
                    world_scene,
                    overlay_scene,
                    "frame graph compile diagnostics resolved"
                );
            }
            return;
        }

        let signature =
            Self::stable_hash(&(world_scene, overlay_scene, registry_revision, diagnostics));
        if self.last_frame_graph_diagnostics_hash == Some(signature) {
            return;
        }
        self.last_frame_graph_diagnostics_hash = Some(signature);

        let first_duplicate_pass = diagnostics
            .duplicate_pass_names
            .first()
            .map(String::as_str)
            .unwrap_or_default();
        let first_missing_dependency = diagnostics
            .missing_dependencies
            .first()
            .map(|(pass, dependency)| format!("{pass}->{dependency}"))
            .unwrap_or_default();

        tracing::warn!(
            world_scene,
            overlay_scene,
            registry_revision,
            issue_count = diagnostics.issue_count(),
            empty_pass_name_count = diagnostics.empty_pass_name_count,
            duplicate_pass_count = diagnostics.duplicate_pass_names.len(),
            missing_dependency_count = diagnostics.missing_dependencies.len(),
            no_registered_passes = diagnostics.no_registered_passes,
            first_duplicate_pass,
            first_missing_dependency,
            "frame graph compile diagnostics"
        );
    }

    fn log_missing_executors_once(&mut self, missing_executors: &[(String, String)]) {
        if missing_executors.is_empty() {
            if self.last_missing_executors_hash.take().is_some() {
                tracing::info!("frame graph executor bindings resolved");
            }
            return;
        }

        let mut unique_missing = missing_executors.to_vec();
        unique_missing.sort();
        unique_missing.dedup();

        let signature = Self::stable_hash(&unique_missing);
        if self.last_missing_executors_hash == Some(signature) {
            return;
        }
        self.last_missing_executors_hash = Some(signature);

        let first_missing = unique_missing
            .first()
            .map(|(pass, executor)| format!("{pass}->{executor}"))
            .unwrap_or_default();
        tracing::warn!(
            missing_count = unique_missing.len(),
            first_missing,
            "frame graph pass executor bindings are missing; skipped pass encoding"
        );
    }

    fn log_execution_order_error_once(&mut self, err: &anyhow::Error) {
        let err_text = err.to_string();
        let signature = Self::stable_hash(&err_text);
        if self.last_execution_order_error_hash == Some(signature) {
            return;
        }
        self.last_execution_order_error_hash = Some(signature);
        tracing::error!(
            error = err_text,
            "frame graph execution order failed; using fallback order"
        );
    }

    fn clear_execution_order_error(&mut self) {
        if self.last_execution_order_error_hash.take().is_some() {
            tracing::info!("frame graph execution ordering recovered");
        }
    }

    fn prepare_registered_passes(
        &mut self,
        device: &Device,
        queue: &Queue,
        frame_data: &RenderFrameDataRegistry<'_>,
        packet: &RendererPreparedPacket,
        active_executors: &BTreeSet<String>,
        render_executor_registry: &RenderPassExecutorRegistryResource,
        timings: &mut RendererFrameTimings,
    ) {
        for executor_name in active_executors {
            if let Some(builtin) = render_executor_registry.resolve_builtin(executor_name) {
                let executor = Self::builtin_pass_executor(builtin);
                executor.prepare(self, device, queue, packet, timings);
                continue;
            }
            if let Some(custom) = render_executor_registry.resolve_custom(executor_name) {
                let mut dispatch_builtin = |builtin: BuiltinRenderPassExecutor| -> Result<()> {
                    let executor = Self::builtin_pass_executor(builtin);
                    executor.prepare(self, device, queue, packet, timings);
                    Ok(())
                };
                let mut ctx = RenderPassPrepareContext::new(
                    device,
                    queue,
                    frame_data,
                    packet.surface_format,
                    packet.surface_size,
                )
                .with_builtin_dispatch(&mut dispatch_builtin);
                if let Err(err) = custom.prepare(&mut ctx) {
                    tracing::error!(
                        executor = executor_name,
                        ?err,
                        "custom render pass executor prepare failed"
                    );
                }
            }
        }
    }

    fn resolve_registered_pipeline(
        &self,
        pass_name: &str,
        pipeline_ref: Option<&RegisteredPipelineRef>,
        named_pipelines: &BTreeMap<String, PipelineKey>,
    ) -> PipelineKey {
        if let Some(pipeline_ref) = pipeline_ref {
            match pipeline_ref {
                RegisteredPipelineRef::Builtin(key) => return key.clone(),
                RegisteredPipelineRef::Named(name) => {
                    if let Some(key) = named_pipelines.get(name).cloned() {
                        return key;
                    }
                    tracing::warn!(
                        pass = pass_name,
                        pipeline_id = name,
                        "registered named pipeline id not found; falling back to pass id key"
                    );
                }
            }
        }

        PipelineKey::from(pass_name.to_string())
    }

    fn resolved_registered_descriptors(
        &self,
        render_graph_registry: &RenderGraphRegistryResource,
    ) -> Vec<ResolvedFramePassDescriptor> {
        let owners = render_graph_registry.owners();
        let mut named_pipelines = BTreeMap::<String, PipelineKey>::new();
        for owner in &owners {
            for pipeline in &owner.pipelines {
                let pipeline_id = pipeline.id.trim();
                if pipeline_id.is_empty() {
                    tracing::warn!(
                        owner = owner.owner,
                        "registered named pipeline has empty id; skipping"
                    );
                    continue;
                }
                if let Some(previous) =
                    named_pipelines.insert(pipeline_id.to_string(), pipeline.key.clone())
                {
                    tracing::warn!(
                        owner = owner.owner,
                        pipeline_id,
                        previous_pipeline = previous.label(),
                        new_pipeline = pipeline.key.label(),
                        "registered named pipeline id replaced previous registration"
                    );
                }
            }
        }

        let mut out = Vec::new();
        for owner in &owners {
            for pass in &owner.passes {
                let pass_name = pass.id.trim();
                if pass_name.is_empty() {
                    tracing::warn!(
                        owner = owner.owner,
                        "registered render pass has empty id; skipping"
                    );
                    continue;
                }
                let kind = match pass.kind {
                    RegisteredPassKind::Compute => PassKind::Compute,
                    RegisteredPassKind::Render => PassKind::Render,
                };
                let pipeline = self.resolve_registered_pipeline(
                    pass_name,
                    pass.pipeline.as_ref(),
                    &named_pipelines,
                );
                let executor = pass
                    .executor
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or(pass_name)
                    .to_string();
                out.push(ResolvedFramePassDescriptor {
                    name: pass_name.to_string(),
                    kind,
                    pipeline,
                    reads: pass
                        .reads
                        .iter()
                        .map(|value| value.trim())
                        .filter(|value| !value.is_empty())
                        .map(str::to_string)
                        .collect(),
                    writes: pass
                        .writes
                        .iter()
                        .map(|value| value.trim())
                        .filter(|value| !value.is_empty())
                        .map(str::to_string)
                        .collect(),
                    depends_on: pass
                        .depends_on
                        .iter()
                        .map(|value| value.trim())
                        .filter(|value| !value.is_empty())
                        .map(str::to_string)
                        .collect(),
                    executor,
                });
            }
        }

        out
    }

    fn build_frame_graph_from_descriptors(
        &self,
        descriptors: &[ResolvedFramePassDescriptor],
    ) -> FrameGraphBuildOutput {
        let mut graph = FrameGraph::new();
        let mut handles = Vec::with_capacity(descriptors.len());
        let mut by_name = BTreeMap::<String, PassHandle>::new();
        let mut pass_executor_bindings = BTreeMap::<String, String>::new();
        let mut diagnostics = FrameGraphCompileDiagnostics::default();

        for descriptor in descriptors {
            let pass_name = descriptor.name.trim();
            if pass_name.is_empty() {
                diagnostics.empty_pass_name_count =
                    diagnostics.empty_pass_name_count.saturating_add(1);
                continue;
            }
            if by_name.contains_key(pass_name) {
                diagnostics.duplicate_pass_names.push(pass_name.to_string());
                continue;
            }

            let mut builder = match descriptor.kind {
                PassKind::Compute => {
                    graph.compute_pass(pass_name.to_string(), descriptor.pipeline.clone())
                }
                PassKind::Render => {
                    graph.render_pass(pass_name.to_string(), descriptor.pipeline.clone())
                }
            };
            if !descriptor.reads.is_empty() {
                builder = builder.reads(descriptor.reads.clone());
            }
            if !descriptor.writes.is_empty() {
                builder = builder.writes(descriptor.writes.clone());
            }
            for dep_name in &descriptor.depends_on {
                let dep_name = dep_name.trim();
                if dep_name.is_empty() {
                    continue;
                }
                if let Some(dep_handle) = by_name.get(dep_name).copied() {
                    builder = builder.depends_on(dep_handle);
                } else {
                    diagnostics
                        .missing_dependencies
                        .push((pass_name.to_string(), dep_name.to_string()));
                }
            }

            let handle = builder.build();
            by_name.insert(pass_name.to_string(), handle);
            pass_executor_bindings.insert(pass_name.to_string(), descriptor.executor.clone());
            handles.push(handle);
        }

        FrameGraphBuildOutput {
            graph,
            handles,
            pass_executor_bindings,
            diagnostics,
        }
    }

}
