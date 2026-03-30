---
title: "Cavern Hunt GDD"
description: "Documentation for Cavern Hunt GDD."
---

# Cavern Hunt GDD

## Overview

`Cavern Hunt` is a co-op 3D SDF cavern expedition game focused on:

- exploring dangerous cave networks with friends
- fighting monsters and elite threats
- looting gear, relics, and materials
- deciding when to push deeper and when to extract

The intended presentation is stylized 3D SDF rendering with a readable fixed-camera or soft-isometric camera model.

## High Concept

Players enter a hostile cavern system, hunt monsters, discover hidden chambers, gather loot, and either extract safely or risk a deeper push for better rewards.

The game should feel like:

- a shared expedition
- a dangerous treasure hunt
- a monster-filled underworld run

It should not feel like:

- a pure wave survival mode
- a passive sandbox
- a slow management sim

## Player Fantasy

The player fantasy is:

- descend into unknown caverns
- survive coordinated monster fights
- uncover secrets and rare finds
- leave with meaningful loot
- return stronger for the next run

The social fantasy is:

- explore together
- call out threats and loot
- split risk and roles
- argue about whether to go deeper or leave

## Pillars

1. **Expedition tension**
   Each run should feel like a dangerous trip into the unknown, not just a sequence of arena fights.

2. **Monster-driven gameplay**
   The primary moment-to-moment gameplay is fighting creatures, elites, and nest threats.

3. **Loot with meaning**
   Drops should change the run or feed longer-term build/progression decisions.

4. **Readable SDF 3D identity**
   The game should look distinct and atmospheric without requiring a heavy traditional art pipeline.

5. **Strong co-op decisions**
   Group movement, target focus, retreat calls, and route choices should matter.

## Target Shape

- Genre: co-op PvE action expedition / loot run
- Camera: fixed-camera 3D or soft-isometric 3D
- Visual style: abstract but atmospheric SDF cavern spaces
- Session style: short-to-medium runs
- Multiplayer target: 4 players first, scalable toward 6-8 later if performance/gameplay supports it
- Authority model: dedicated server authority

## Core Loop

1. Enter a cavern instance.
2. Explore branching tunnels and chambers.
3. Fight monster packs, elites, and nest encounters.
4. Collect loot, crafting materials, and relics.
5. Decide whether to:
   - extract now, or
   - push deeper for better rewards and higher danger.
6. Return, apply rewards/upgrades, and launch the next run.

## Run Structure

Each run should contain:

- a start/drop zone
- a sequence of connected cavern spaces
- optional side rooms
- at least one elite or nest encounter
- a clear extraction path or final objective

Possible run shapes:

- linear descent with side branches
- hub chamber with branching sub-routes
- layered cavern where deeper levels are riskier and more rewarding

## Exploration

Exploration should be meaningful, not filler.

Exploration value should come from:

- hidden loot rooms
- forked paths with different threat/reward profiles
- monster nests
- environmental landmarks
- rare event rooms
- alternate extraction routes

Exploration should create small team decisions:

- split or stay together
- clear side room or keep tempo
- spend resources on a risky route or save them

## Combat

Combat direction:

- real-time
- readable in multiplayer chaos
- mobility matters
- elites must change how the room is played

First combat target:

- basic attack
- dodge or movement burst
- one utility or class ability
- enemy telegraphs
- clear hit feedback

Encounter types:

- roaming packs
- ambush rooms
- nest waves
- elite guardians
- chamber boss or extraction holdout

## Monsters

Monster design should favor strong silhouettes and behavior contrast.

Early enemy roles:

- **Swarmers**: fast, low-health pressure units
- **Bruisers**: slower front-line threats
- **Spitters**: ranged denial / projectile pressure
- **Nest Core**: static or semi-static encounter anchor
- **Elite Variant**: upgraded behavior and better loot

SDF is especially well suited to:

- blobs
- segmented creatures
- crystalline monsters
- fungal or insectoid silhouettes
- glow-based weak points

## Loot

Loot is central.

The first loot model should be simple but exciting:

- common drops from normal enemies
- better drops from elites and nests
- rare relics from hidden rooms or encounter finales

Early loot categories:

- weapons
- ability modifiers
- stat charms
- relics
- crafting materials

Loot should support:

- immediate run power
- long-term progression
- clear rarity and identity

## Progression

Two progression layers are recommended.

### In-run progression

- temporary weapons/upgrades
- found relics
- consumables
- encounter rewards

### Between-run progression

- unlocks
- class/loadout growth
- persistent gear slots
- crafting or upgrade spend

The MVP can start with a very light between-run layer if needed.

## Multiplayer

Primary model:

- dedicated server authority
- 4-player target for first playable slice
- reconnect support
- snapshot replication
- replay/checkpoint support on the server

Multiplayer goals:

- friends can join a run and stay in sync reliably
- combat state is authoritative on the server
- reconnect should restore the player into an active session cleanly
- group play should be more fun than solo, not just easier

## SDF 3D Direction

The renderer should support the game’s identity by emphasizing:

- strong cavern silhouettes
- readable enemy shapes
- glowing loot/relics
- atmospheric fog/light gradients
- simple but striking hazards

Recommended camera approach:

- fixed 3D exploration/combat camera
- slight tilt for depth readability
- stable composition in multiplayer fights

Avoid for the first version:

- fully free third-person camera
- heavy cinematic camera work
- over-detailed material complexity

### Material/Lighting Direction

The current renderer direction combines:

- asset-authored material graphs (`RON`)
- triplanar procedural surface generation (no UV seams)
- PBR-lite direct lighting (GGX/Smith/Schlick)
- staged GI:
  - `Off`
  - `AO + bent normal ambient`
  - probe-based GI (incremental follow-on)

This keeps the look flexible and data-driven without requiring a heavy content pipeline.

Current baseline profile target:

- default look-dev profile: `balanced`
- default runtime mode: `material_graph`
- fallback mode for safety: `legacy`

## MVP Slice

The first playable `Cavern Hunt` slice should include:

- one cavern biome
- one run flow
- 3 normal monster types
- 1 elite type
- 1 extraction or boss finale
- 4-player co-op target
- a small loot pool
- one simple persistent reward loop

## Current Vertical Slice

The current implementation target is a friend-testable local/dev slice:

- fixed soft-isometric SDF 3D camera
- one procedural cavern layout per run
- one hunter archetype
- ranged primary fire
- dash
- `Swarmer`, `Bruiser`, `Spitter`, and `NestGuardian`
- elite kill unlocks extraction
- 1 to 4 live clients on a dedicated-authority server
- AI fill for missing slots
- reconnect into the same live run
- local `CavernMarks` rewards on successful extraction

This slice is intended to prove:

- the run is readable and finishable by real players
- multiple clients can share the same run reliably
- the SDF presentation works for real moment-to-moment combat, not just an example scene

That slice should prove:

- exploration is fun
- monsters are fun to fight
- loot is satisfying
- the game feels good with friends

## Technical Fit With Current Project

This game direction fits the current runtime well because the project already has:

- dedicated-authority networking
- QUIC session runtime
- reconnect support
- replay/checkpoint infrastructure
- scene snapshot replication
- SDF renderer groundwork

The main gameplay work still needed is:

- actual monster AI/runtime
- richer player combat
- real loot systems
- route/objective generation
- run/session gameplay rules

## Risks

Main risks:

- too little encounter variety makes exploration feel empty
- too much abstraction in visuals could hurt combat readability
- loot can feel shallow if rarity and identity are weak
- exploration can become downtime if rewards are too sparse

The first versions must bias toward:

- readable monsters
- compact runs
- frequent small rewards
- clear team decisions

## Near-Term Design Goal

Build a first vertical slice where 4 players can:

- enter one cavern
- explore connected rooms
- fight several monster packs and one elite encounter
- find loot
- extract successfully

If that slice feels good, the project has the right game direction.
