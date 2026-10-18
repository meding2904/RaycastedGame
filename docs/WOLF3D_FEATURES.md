# Feature matrix: Raycasted Wolf vs Wolfenstein 3D (1992)

`yes` means the feature exists and is exercised by the tests or the frame harness. `n/a` means the
original had it but this game deliberately does not, with the reason given.

## Presentation

| Feature | Status | Notes |
| --- | --- | --- |
| 320x200 internal resolution | yes | `FRAME_WIDTH`/`FRAME_HEIGHT`; the view occupies the top 160 rows |
| Nearest-neighbour upscale | yes | letterboxed to 16:10 in any window size, integer scaling where it fits |
| Textured walls | yes | 12 wall textures, one per tile |
| Textured floors and ceilings | yes | flat per-tile textures, per-region overrides |
| Distance fog | yes | the engine's fog, tuned for a 6 tile falloff |
| Sprite billboards with depth | yes | z-buffered against the wall columns |
| Weapon overlay with muzzle flash | yes | per-weapon animation frames, full-view flash on firing |
| Animated status bar face | yes | reacts to health, recent damage, firing |
| Readable 5x7 bitmap font | yes | one font for the bar and every screen, drawn with a hard shadow |
| 70 Hz fixed timestep with frame pacing | yes | accumulator, capped catch-up, window refresh paced to the tick |

## Levels and structures

| Feature | Status | Notes |
| --- | --- | --- |
| 64x64 tile maps | yes | `MAP_WIDTH`/`MAP_HEIGHT`, three floors |
| Sliding doors | yes | texture slides into the wall frame; passable at 75% open |
| Auto-closing doors | yes | 5 seconds, and never on top of the player or an enemy |
| Gold and silver key doors | yes | refuse with a lock sound and a message until the key is held |
| Pushwalls | yes | two tiles, crushing anything in the way, revealing a secret |
| Secret areas | yes | rectangular regions, counted in the tally, reachable once the wall moves |
| Elevator switch ends the floor | yes | `Wall::ElevatorSwitch`, triggers the fade and the tally |
| Level validation | yes | doors, keys and secrets are checked for reachability at load; a broken floor fails the tests |
| Original episode layouts | n/a | the 1992 maps are id's data; these are original layouts in the same style |
| Level editor | n/a | the layout DSL in `levels.rs` is the editor; an in-game one is out of scope |

## Player

| Feature | Status | Notes |
| --- | --- | --- |
| Free 8-direction movement with collision | yes | axis-separated sliding against walls and closed doors |
| Arrows turn or walk | yes | left/right turn, up/down walk |
| Strafing | yes | `A`/`D` |
| Running | yes | `Shift`, 2.8 vs 5.0 tiles per second |
| Mouse turn | yes | relative motion, sensitivity in `player.rs` |
| Use key | yes | doors, pushwalls, elevator |
| Health, damage, pain flash | yes | 100 max, short invulnerability after a hit |
| Lives and respawn | yes | three lives, respawn at the floor start, floor restarts |
| Death animation and game over | yes | view drops, then game over |
| Score | yes | treasure, kills, and secrets all pay out |
| Keys in inventory | yes | shown in the status bar |
| God mode | n/a | a cheat; out of scope, and the face has a branch ready for it |
| Save/load | n/a | out of scope, listed in the design |

## Weapons

| Weapon | Damage | Rate | Ammo | Status |
| --- | --- | --- | --- | --- |
| Knife | 5-10 | 0.45 s | none | yes |
| Pistol | 8-16 | 0.42 s | yes | yes, starting weapon |
| Machine gun | 8-16 | 0.16 s | yes | yes, found on the floor |
| Chaingun | 8-16 | 0.09 s | yes | yes, in a secret armoury |

All four use hitscan along the exact centre line, which respects walls, closed doors and enemy
bodies, an empty-clip click, and a per-weapon animation and sound.

## Enemies

| Type | Health | Speed | Damage | Behaviour |
| --- | --- | --- | --- | --- |
| Guard | 25 | 2.3 | 8 | patrols, fires |
| SS | 100 | 2.6 | 14 | fires, tougher |
| Officer | 50 | 3.6 | 10 | fastest shooter |
| Guard dog | 20 | 4.4 | 6 | melee only |
| Boss | 400 | 1.9 | 24 | fires fast, huge |

States: idle, patrol, chase, attack, pain, dying, dead. Enemies wake on sight with a line of sight
check, or by hearing a shot, a shout or a door. Ambush guards wait until they see the player.
Guards drop ammo on death. Everything scales with the difficulty tier (damage 0.25x to 1.5x, health
up to 1.25x).

Attacks are animated in four frames: raise, aim, fire, recover. The damage lands on the fire frame
about a third of a second after the enemy commits, and the line of sight is checked again at that
moment, so a player who reaches a door in time is not hit. The dog has its own four frame lunge in
the same slots. Every sprite is drawn with a lit and a shadowed edge per material and gets a dark
outline, so enemies read against the wall behind them.

## Pickups and decoration

Health (small and large), ammo (clip and box), four treasure types with different values, gold and
silver keys, an extra life, machine gun and chaingun pickups, barrels, tables, pillars, lamps,
chandeliers, plants and blood decals. Pickups are solid to the player, taken on touch, and their
values scale with difficulty.

## Flow and screens

Difficulty screen with the original's four tiers and wording; floor intro card; pause; death;
floor complete; kill/secret/treasure/time tally with percentage bonuses and the score rolling up;
game over; victory after the last floor; restart from any of them.

## Sound

24 synthesised effects (four weapons, four enemy voices, doors, pushwall, pickups, keys, death,
elevator, pain) and a looping 14 second music bed, both generated at startup into memory. A machine
without an audio device gets silence, a log line and a working game.

## Deliberate omissions

Network play, save games, the console level editor, AdLib/OPL2 emulation, the retail episodes' 60
floors, DOS-exact par times, and any id artwork, sound or level data.
