use cavern_hunt::{CavernRunConfig, CavernSeed, domain::CavernLayout};
use std::collections::{HashSet, VecDeque};

#[test]
fn same_seed_generates_identical_layout() {
    let config = CavernRunConfig::default();
    let a = CavernLayout::generate(CavernSeed(42), &config);
    let b = CavernLayout::generate(CavernSeed(42), &config);
    assert_eq!(a, b);
}

#[test]
fn different_seed_changes_layout() {
    let config = CavernRunConfig::default();
    let a = CavernLayout::generate(CavernSeed(42), &config);
    let b = CavernLayout::generate(CavernSeed(99), &config);
    assert_ne!(a, b);
}

#[test]
fn layout_guarantees_elite_and_extraction_reachability() {
    let config = CavernRunConfig::default();
    let layout = CavernLayout::generate(CavernSeed(77), &config);
    let adjacency = layout.adjacency();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::from([layout.start_room]);
    while let Some(room) = queue.pop_front() {
        if !visited.insert(room) {
            continue;
        }
        if let Some(neighbors) = adjacency.get(&room) {
            for neighbor in neighbors {
                queue.push_back(*neighbor);
            }
        }
    }
    assert!(visited.contains(&layout.elite_room));
    assert!(visited.contains(&layout.extraction_room));
}

#[test]
fn layout_contains_optional_branch_and_loot_room() {
    let config = CavernRunConfig::default();
    let layout = CavernLayout::generate(CavernSeed(1234), &config);
    let has_loot = layout
        .rooms
        .iter()
        .any(|room| room.role == cavern_hunt::domain::RoomRole::Loot);
    assert!(has_loot);
    assert!(layout.rooms.len() >= 7);
    assert!(layout.connections.len() >= layout.rooms.len().saturating_sub(1));
}
