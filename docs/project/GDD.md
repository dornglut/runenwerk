# Grotto Quest - Game Design Document

## 1. Vision
Grotto Quest is a 3D hack-and-slash dungeon crawler focused on party control, procedural replayability, and high-expression combat/build crafting.

## 2. Design Pillars
- Customizability, expression, and meaningful player choice.
- Endless replayability through procedural variation and long-term growth.
- Learn-by-doing tutorial philosophy: show, do not tell.
- Skillful adaptation over brainless repetition.

## 3. Inspirations
- Dragon Quest 9
- Disgaea
- Diablo 3
- Last Epoch
- Rune Factory

## 4. Core Concept
- 3D action combat.
- Hack-and-slash enemy density.
- Party management and companion control.
- Procedural dungeon mapping and progression.

## 5. Core Features
- Character creation.
- Procedural dungeons.
- Skills and abilities.
- Real-time combat.
- Partner companions.
- Hub preparation between dungeon runs.

## 6. Game Start
- Player creates main character.
- Player is dropped directly into a dungeon.
- Immediate survival loop begins.

## 7. Tutorial Policy
- Show actions through allies, enemies, and encounter scripting.
- Communicate possible actions through gameplay situations.
- If the player saw it work, the player should be able to do it.
- Teach by exposure and repetition in context.

## 8. Core Gameplay Loop
- Enter dungeon.
- Fight and explore.
- Collect loot.
- Defeat boss.
- Return to hub.
- Prepare, craft, customize, and adjust squad strategy.
- Re-enter deeper dungeon tier.
- Repeat until end boss.
- Prestige reset unlocks additional systems and options.

## 9. Combat Design
- High enemy volume hack-and-slash pacing.
- Solo and team control options.
- Player can issue ally behavior commands.
- Allies can operate autonomously.
- Player can swap controlled party member for tactical moments.
- Skill use, rotations, and positioning matter.
- Enemy telegraphs encourage reaction and adaptation.

## 10. Progression Design
- Difficulty and complexity onboarding should be player-adjustable.
- Endgame systems should be available early in lighter form.
- Growth should be knowledge-based and skill-based.
- Early systems are simple; later systems layer depth.
- Support both low-complexity and high-complexity builds.

## 11. Variation and Challenge Rules
- No overfitting to one strategy.
- No thoughtless rushing without consequences.
- Environments and enemy archetypes should demand adaptation.
- Repetitive spamming should reduce efficiency or create risk.
- Reward fast thinking and situational decisions.

## 12. MVP Scope
- Character creation: name, traits/perks, basic look.
- Combat: attack, dodge, simple combos, 2 skills per profession.
- Procedural dungeon: 3-5 rooms, simple enemy pool, 1 boss.
- Loot: weapons and armor with basic stat variation.
- Party: 1 companion with simple autonomous behavior.
- Progression: XP, levels, basic skill unlocks, core stats.
- Hub: one area with shop and storage.

## 13. MVP Loop
- Enter procedural dungeon.
- Clear 3-5 encounters.
- Loot and gain XP.
- Defeat mini-boss.
- Return to hub to equip and prepare.
- Repeat.

## 14. Character and Party Creation
- Main character: traits (positive and negative), appearance.
- Party members: recruit/create and configure behavior.
- Dungeon progression supports cooperative squad identity.

## 15. Skill System Direction
- Skills are runtime-composed ECS entities.
- Behaviors come from modular components (damage, projectile, AoE, modifiers, visuals, cooldown, cost).
- Runtime modifiers from items/perks/buffs should add or remove components.
- Avoid skill blueprint explosion by using component composition.
