use editor_preview::{ReloadDecision, ReloadStatus, ReloadSubject, ReloadSubjectKind};
use engine::plugins::render::shader::ReloadStatusPayload;

pub fn shader_reload_status_to_preview_status(payload: &ReloadStatusPayload) -> ReloadStatus {
    let decision = if payload.error.is_some() {
        ReloadDecision::FailedPreserved
    } else if matches!(payload.state.as_str(), "loaded" | "reloaded") {
        ReloadDecision::LiveReload
    } else {
        ReloadDecision::PreviewSessionRestartRequired
    };
    let mut status = ReloadStatus::new(
        ReloadSubject::new(ReloadSubjectKind::Shader, payload.id.clone()),
        decision,
        shader_reload_message(payload, decision),
    );
    if let Some(error) = &payload.error {
        status = status.with_diagnostic(error.clone());
    }
    if let Some(details) = &payload.details {
        status = status.with_diagnostic(details.clone());
    }
    status
}

fn shader_reload_message(payload: &ReloadStatusPayload, decision: ReloadDecision) -> String {
    match decision {
        ReloadDecision::LiveReload => format!("shader {} is live-reloadable", payload.id),
        ReloadDecision::FailedPreserved => {
            format!(
                "shader {} failed reload; preserving prior valid shader",
                payload.id
            )
        }
        ReloadDecision::PreviewSessionRestartRequired => {
            format!("shader {} requires a preview-session refresh", payload.id)
        }
        ReloadDecision::RuntimeProcessRestartRequired => {
            format!("shader {} requires a runtime-process restart", payload.id)
        }
        ReloadDecision::Unsupported => format!("shader {} reload is unsupported", payload.id),
        ReloadDecision::Rejected => format!("shader {} reload was rejected", payload.id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_status_maps_loaded_to_live_reload() {
        let payload = ReloadStatusPayload::new(
            "shader",
            "main",
            "loaded",
            "assets/shaders/main.wgsl",
            2,
            true,
            None,
            None,
            None,
        );

        let status = shader_reload_status_to_preview_status(&payload);

        assert_eq!(status.decision, ReloadDecision::LiveReload);
        assert_eq!(status.subject.kind, ReloadSubjectKind::Shader);
    }

    #[test]
    fn shader_status_maps_error_to_failed_preserved() {
        let payload = ReloadStatusPayload::new(
            "shader",
            "main",
            "error",
            "assets/shaders/main.wgsl",
            2,
            true,
            None,
            Some("compile failed".to_string()),
            None,
        );

        let status = shader_reload_status_to_preview_status(&payload);

        assert_eq!(status.decision, ReloadDecision::FailedPreserved);
        assert_eq!(status.diagnostics, vec!["compile failed".to_string()]);
    }
}
