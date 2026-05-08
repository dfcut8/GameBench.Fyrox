# 2D Benchmark Game Recreation Plan

## Summary

This game is a desktop 2D benchmark. It opens a fixed-size window, shows a simple main menu, and then runs a benchmark scene filled with many animated pixel-art objects floating inside a large 2D world. The player can increase or decrease the target object count while watching the current FPS.

The goal of this document is to describe the game itself so it can be recreated in another project with a different engine or technology stack. It intentionally avoids engine-specific, language-specific, dependency, and build instructions.

## Window, World, And Camera

- Window resolution is fixed at `1280x960`.
- The playable benchmark world is `3200x3200` pixels.
- The benchmark camera is zoomed out enough to show the full `3200` pixel world height.
- At the `1280x960` window aspect ratio, the camera sees about `4267x3200` world pixels, so the full world height is visible and there is extra horizontal space.
- The world origin is centered in the playable area.
- The world bounds are solid walls around the full `3200x3200` area.
- Moving benchmark objects are clamped inside the world bounds.
- Visuals should preserve a pixel-art look. Sprites and tiles should be rendered with crisp nearest-neighbor-style sampling where possible.

## Main Menu

The game starts on a main menu. Benchmark objects are not spawned until the benchmark starts.

- Background: solid very dark blue-black.
- Title text: `KAIJU 2D BENCHMARK`.
- Menu options:
  - `Start Benchmark`
  - `Quit`
- Options appear as simple light rectangular buttons centered in the menu area.
- Pointer or mouse click can activate either option.
- Keyboard controls:
  - `Enter`: start benchmark.
  - `Q`: quit.

## Benchmark Scene

Starting the benchmark switches from the main menu to the benchmark scene.

- Default target object count is `100`.
- Current object count ramps toward the target count.
- At most `25` new objects are spawned per frame.
- If the target count is reduced, extra objects are removed immediately from the end of the active object list.
- Target object count range is `0` to `5000`.
- Target object count changes in steps of `100`.

Benchmark controls:

- `A`: increase target object count by `100`.
- `Z`: decrease target object count by `100`.
- `R`: reset or reload benchmark to the default `100` object target.
- `Q`: quit.

Reset behavior:

- Destroy all current benchmark objects.
- Clear the current object list.
- Reset target object count to `100`.
- Reset the smoothed FPS display state.
- Spawn objects again using the normal ramp-up behavior.
- Keep the benchmark scene, background, controls, and music active.

## Benchmark Objects

Each benchmark object is an animated `32x32` pixel-art square/blob sprite.

- Render size: `32x32` pixels.
- Collision size: `32x32` axis-aligned box.
- No gravity.
- No rotation.
- No depth changes during gameplay.
- Objects float freely in 2D space.
- Objects are rendered above the tilemap background.
- Each object uses one of the animation frames as its starting frame.
- Each object starts with a randomized animation timer so all objects do not animate in perfect sync.

Spawn rules:

- Spawn position is random inside the `3200x3200` world.
- Spawn position must leave at least one sprite size of margin from the world edges.
- Initial horizontal velocity is random from `-140` to `140` pixels per second.
- Initial vertical velocity is random from `-140` to `140` pixels per second.
- If the combined absolute velocity is too slow, specifically less than `80`, replace it with `120` horizontal and `80` vertical pixels per second.

Animation rules:

- Animation has `8` frames.
- Frame duration is `0.125` seconds.
- Animation loops forever.
- Sprite positions are rounded to whole pixels before drawing.

Collision rules:

- Each object moves by velocity each frame.
- When an object hits a world edge, clamp it inside the bounds and reverse the velocity component for that axis.
- Object-object collision uses axis-aligned `32x32` boxes.
- When two objects overlap:
  - Compute overlap on X and Y.
  - Separate them along the axis with the smaller overlap.
  - Move each object half of the overlap distance away from the other.
  - Swap the velocity components on the collision axis.
  - Trigger the collision flash on both objects.
- Collision detection should be broad-phased with a grid or equivalent partitioning so thousands of objects remain practical.
- A grid cell size of `32` pixels matches the object size.
- For each object, check its own grid cell and the eight neighboring cells.

Collision flash:

- Flash duration is `0.25` seconds.
- During the flash, the object is visibly brighter than normal.
- After the flash timer expires, the object returns to its normal color.

## Sprite Art Direction

The moving object sprite is a small pixel-art blob inside a `32x32` square.

Required traits:

- The art should clearly read at `32x32`.
- It should have a dark outline.
- It should have a colorful body.
- It should include simple face details such as eyes, a mouth, highlights, and small cheek pixels.
- It should animate with a subtle bobbing or squashing motion.
- The animation should use `8` frames.

The existing visual target uses four color families across the frames:

- Blue/cyan.
- Green/mint.
- Purple/magenta.
- Orange/yellow.

The animation may cycle through these color families, or objects may start on different frames, producing a mixed-color field.

## Tilemap Background

The benchmark scene has a `3200x3200` pixel tilemap background behind all benchmark objects.

- Tiles are `32x32` pixels.
- The complete world is filled by a `100x100` tile grid.
- The tilemap should be visually varied, not a single flat color.
- The main base is grass.
- Add clear variation using:
  - Grass variants.
  - Dirt paths.
  - Stone or rocky patches.
  - Water areas.
  - Flowers.
  - Rocks.
  - Bushes.
- The tilemap must render behind moving objects.
- The tilemap should stay fixed in world space.

Suggested layout:

- Mostly grass across the full map.
- A visible river or water section near one side or corner.
- A vertical or winding path crossing part of the map.
- A dirt clearing in one region.
- Scattered decorative flowers, rocks, and bushes across the grass.
- Avoid high-frequency noise that distracts from the benchmark objects.

## Overlay UI

During the benchmark scene, a readability panel appears in the top-left corner of the window.

- Panel position: top-left with a small margin.
- Panel color: solid very dark background.
- Panel opacity: fully opaque.
- Text color: white.
- Text should be large enough to read during motion.
- The panel should not block input.

Overlay text format:

```text
FPS: <fps>
Objects: <current>/<target>
A add | Z remove | R reset | Q quit
```

FPS behavior:

- FPS is computed from frame time.
- Displayed FPS is smoothed.
- Use a smoothing behavior equivalent to keeping 90% of the previous displayed FPS and adding 10% of the newest instantaneous FPS.
- Update the displayed text every `0.25` seconds.
- Round displayed FPS to a whole number.

## Audio

The benchmark includes simple looping background music.

- Style: short 8-bit or chiptune loop.
- Loop length: about `8` seconds.
- The loop should play continuously during the game.
- The loop should be unobtrusive and suitable for long benchmark runs.

The current target sound is built from:

- Square-wave lead notes.
- A simple bass pattern.
- A small rhythmic click or pulse.
- Mono output is acceptable.
- `44.1 kHz`, `16-bit` audio is acceptable.

## Logging And Observability

The game should provide basic runtime logs to standard output or the platform's normal developer console.

Important events to log:

- Game launch.
- Camera or viewport setup.
- Main menu shown.
- Benchmark started.
- Tilemap created or loaded.
- Background music started.
- Target object count changes.
- Benchmark reset.
- Quit selection.
- Object spawn progress for large counts.

Logs are for development and troubleshooting only. They do not appear in the game UI.

## Acceptance Checklist

- The window opens at `1280x960`.
- The game starts on the main menu.
- `Start Benchmark` and `Enter` begin the benchmark.
- `Quit` and `Q` exit the game from the menu.
- Benchmark starts with target count `100`.
- Moving objects spawn gradually, up to `25` per frame.
- `A` increases target by `100`.
- `Z` decreases target by `100`.
- Target count never goes below `0` or above `5000`.
- `R` removes all current objects and reloads the benchmark to `100`.
- `Q` exits from the benchmark scene.
- Objects are `32x32`, animated, and visually pixel-art.
- Objects move without gravity.
- Objects collide with world walls and with each other.
- Objects flash for `0.25` seconds after object-object collision.
- Objects remain visually 2D with no rotation.
- The tilemap fills the `3200x3200` world and renders behind sprites.
- The overlay shows FPS, current object count, target object count, and controls.
- Overlay text remains readable over the tilemap and sprites.
- Background music loops continuously.
