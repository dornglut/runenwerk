// Owner: Cavern Hunt Gameplay Plugin - Session Sync
fn sync_session_runtime_config_system(
    session: Res<SessionRuntimeState>,
    mut config: ResMut<CavernRunConfig>,
) -> Result<()> {
    sync_session_runtime_config(&session, &mut config);
    Ok(())
}

fn sync_session_spawn_policy_system(
    session: Res<SessionRuntimeState>,
    config: Res<CavernRunConfig>,
    mut policy: ResMut<SessionSpawnPolicy>,
) -> Result<()> {
    sync_session_spawn_policy(&session, &config, &mut policy);
    Ok(())
}

fn sync_session_runtime_config(session: &SessionRuntimeState, config: &mut CavernRunConfig) {
    if session.max_players > 0 {
        // Local/dev dedicated-authority sessions can admit with a fallback join state
        // that carries `max_players = 1`. Do not collapse the game-configured run
        // capacity in that case; only widen capacity here until real lobby metadata
        // becomes mandatory for all admissions.
        config.max_players = config.max_players.max(session.max_players.max(1));
    }
    if let Some(settings) = parse_cavern_session_settings(session) {
        if let Some(seed) = settings.seed {
            config.seed = seed;
        }
        if let Some(enemy_density) = settings.enemy_density {
            config.enemy_density = enemy_density.max(0.1);
        }
        if let Some(extract_countdown_seconds) = settings.extract_countdown_seconds {
            config.extract_countdown_seconds = extract_countdown_seconds.max(0.0);
        }
        if let Some(base_scrap_reward) = settings.base_scrap_reward {
            config.base_scrap_reward = base_scrap_reward;
        }
    }
}

fn sync_session_spawn_policy(
    session: &SessionRuntimeState,
    config: &CavernRunConfig,
    policy: &mut SessionSpawnPolicy,
) {
    let desired_human_players = session.roster_player_codes.len().clamp(1, u8::MAX as usize) as u8;
    let desired_total_participants = if session.admitted {
        session
            .ai_fill_target
            .max(desired_human_players)
            .min(config.max_players)
    } else {
        1
    };
    let companion_target_count = desired_total_participants.saturating_sub(desired_human_players);
    let settings = parse_cavern_session_settings(session).unwrap_or_default();
    policy.desired_human_players = desired_human_players;
    policy.desired_total_participants = desired_total_participants;
    policy.companion_target_count = companion_target_count;
    policy.spawn_radius = settings.spawn_radius.unwrap_or(1.1).max(0.6);
    policy.companion_spacing = settings.companion_spacing.unwrap_or(1.25).max(0.75);
    policy.roster_display_names = session
        .roster_player_codes
        .iter()
        .enumerate()
        .map(|(index, code)| (index as u8, code.clone()))
        .collect();
    policy.difficulty = RunDifficultyProfile {
        enemy_health_scale: settings.enemy_health_scale.unwrap_or(1.0).max(0.5),
        enemy_damage_scale: settings.enemy_damage_scale.unwrap_or(1.0).max(0.5),
        elite_health_bonus: settings.elite_health_bonus.unwrap_or(0.0).max(0.0),
    };
}

fn parse_cavern_session_settings(session: &SessionRuntimeState) -> Option<CavernSessionSettings> {
    let raw = session.settings_json.as_ref()?;
    serde_json::from_str(raw).ok()
}
