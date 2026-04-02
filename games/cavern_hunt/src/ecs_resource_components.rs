// Owner: Cavern Hunt ecs migration - Resource component markers
//
// Phase 3 aligns resource usage with ecs component-backed resources.
// These impls keep resource marker wiring explicit without changing
// domain type definitions.

impl ecs::Component for crate::domain::gameplay::session::CavernRunConfig {}
impl ecs::Component for crate::domain::gameplay::session::SpawnDirector {}
impl ecs::Component for crate::domain::gameplay::run::CavernRunState {}
impl ecs::Component for crate::domain::gameplay::run::CavernObjectiveState {}
impl ecs::Component for crate::domain::gameplay::run::ExtractionState {}
impl ecs::Component for crate::domain::gameplay::spawn::SessionSpawnPolicy {}
impl ecs::Component for crate::domain::world::worldgen::CavernLayout {}
impl ecs::Component for crate::domain::world::geometry_graph::CavernTopology {}
impl ecs::Component for crate::domain::loot::LootTableRegistry {}
impl ecs::Component for crate::domain::gameplay::encounter::RoomEncounterRegistry {}
impl ecs::Component for crate::domain::gameplay::camera::CavernCameraState {}
impl ecs::Component for crate::domain::gameplay::meta::CavernMetaProfile {}
impl ecs::Component for crate::domain::gameplay::meta::CavernMetaPersistenceConfig {}
impl ecs::Component for crate::domain::gameplay::meta::CavernMetaRewardState {}
impl ecs::Component for crate::domain::gameplay::hud::CavernHudState {}
impl ecs::Component for crate::domain::gameplay::local::LocalPlayerRef {}
impl ecs::Component for crate::domain::gameplay::player_control::CavernAimState {}
impl ecs::Component for crate::domain::gameplay::player_control::CavernControlState {}
impl ecs::Component for crate::domain::gameplay::player_control::CavernPredictionState {}
impl ecs::Component for crate::domain::gameplay::runtime::CavernServerControlMap {}
impl ecs::Component for crate::domain::gameplay::runtime::CavernServerAppliedInputTickMap {}
impl ecs::Component for crate::domain::gameplay::runtime::CavernPlayerOwnershipState {}
impl ecs::Component for crate::domain::render_sdf::CavernSdfWorldFrame {}
impl ecs::Component for crate::domain::material_runtime::CavernMaterialQualityConfig {}
impl ecs::Component for crate::domain::material_runtime::CavernMaterialRegistry {}
impl ecs::Component for crate::domain::material_runtime::CavernMaterialRuntimeState {}
impl ecs::Component for crate::domain::material_runtime::GiProbeGrid {}
impl ecs::Component for crate::domain::material_runtime::GiProbeUpdateQueue {}
impl ecs::Component for crate::domain::gameplay::tuning::PlayerCombatTuning {}
impl ecs::Component for crate::domain::gameplay::tuning::EnemyCombatTuning {}
impl ecs::Component for crate::net::replication::ServerReplicationMap {}
impl ecs::Component for crate::net::replication::ClientReplicationMap {}
impl ecs::Component for crate::net::interpolation::InterpolationConfig {}
impl ecs::Component for crate::net::policy::AdaptiveSmoothingState {}
impl ecs::Component for crate::net::policy::CorrectionStats {}
impl ecs::Component for crate::net::policy::ReplicationRuntimeMetrics {}
impl ecs::Component for crate::net::policy::ReplicationBudgetConfig {}
impl ecs::Component for crate::net::policy::ReplicationCadenceConfig {}
impl ecs::Component for crate::net::policy::ReplicationLoadShedConfig {}
impl ecs::Component for crate::net::policy::ReplicationKeyframeConfig {}
impl ecs::Component for crate::net::config::NetDiagnosticsConfigAssetV1 {}
impl ecs::Component for crate::net::config::ClientNetworkConfigAssetV1 {}
impl ecs::Component for crate::net::config::ServerNetworkConfigAssetV1 {}
impl ecs::Component for crate::net::config::NetConfigHotReloadState {}
