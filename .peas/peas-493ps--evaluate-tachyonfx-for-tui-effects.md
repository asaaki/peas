+++
id = "peas-493ps"
title = "Evaluate tachyonfx for TUI effects"
type = "research"
status = "completed"
priority = "normal"
parent = "peas-7z7f5"
created = "2026-01-19T00:22:55.577970100Z"
updated = "2026-01-19T00:31:55.349861300Z"
+++

## Evaluation of tachyonfx

### What it does
TachyonFX is an animation/effects library for ratatui that provides 40+ visual effects including:
- Color transitions (fade, sweep, HSL animations)
- Text effects (dissolve, coalesce, slide, explode)
- Spatial patterns (radial, diagonal, checkerboard reveals)
- Timing control (parallel, sequential, looping, ping-pong)

### Integration Pattern
```rust
let mut effects: EffectManager<()> = EffectManager::default();
let fx = fx::fade_to(Color::Cyan, Color::Gray, (1_000, Interpolation::SineIn));
effects.add_effect(fx);

// In render loop
effects.process_effects(elapsed.into(), frame.buffer_mut(), screen_area);
```

### Use Cases for peas
Potential applications:
1. Selection bar pulse/glow animation
2. Fade transitions when changing views
3. Highlight flash when items are updated
4. Status change color sweeps

### Verdict: NICE TO HAVE
This is genuinely cool but:
1. Adds complexity and dependency overhead
2. Terminal effects may be distracting for a productivity tool
3. Current UI is functional and clean

**Recommendation**: Consider for v2 or as optional feature. The selection bar blinking effect could be nice but is not essential. Priority: Low.
