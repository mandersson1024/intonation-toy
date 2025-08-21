# Egui Text Rendering System

## Overview

The text rendering system uses egui's font rendering capabilities to display note labels in the three-d graphics context.

## Architecture

### EguiTextBackend

The text rendering is implemented through the `EguiTextBackend` struct, which provides direct text rendering:

```rust
pub struct EguiTextBackend {
    egui_ctx: egui::Context,
    font_texture: Option<Texture2DRef>,
}
```

#### Key Features:
- Direct text rendering without queuing
- Single font texture management
- Roboto font support
- Transparent blending for overlays

## Implementation Details

### Font Setup
- Loads Roboto-Regular.ttf from static resources
- Configures both Proportional and Monospace font families
- Manages font texture updates automatically

### Rendering Pipeline
1. Begin egui pass with viewport dimensions
2. Create text shapes for each text item
3. Tessellate shapes into mesh primitives
4. Update font texture if needed
5. Convert egui meshes to three-d render objects
6. Return render objects for compositing

### Coordinate System
- Egui uses top-left origin
- Three-d uses bottom-left origin
- Y-coordinate flipping is handled during conversion

## Usage

```rust
// Create text backend
let mut text_backend = EguiTextBackend::new()?;

// Prepare text data (text, x, y, size, color)
let texts = vec![
    ("C4".to_string(), 10.0, 100.0, 14.0, [1.0, 1.0, 1.0, 1.0]),
    ("D4".to_string(), 10.0, 120.0, 14.0, [1.0, 1.0, 1.0, 1.0]),
];

// Render texts and get render objects
let render_objects = text_backend.render_texts(&context, viewport, &texts);

// Render objects can be drawn to screen
for obj in &render_objects {
    screen.render(&camera, [obj.as_ref()], &[]);
}
```

## Integration with MainScene

The MainScene uses EguiTextBackend to render note labels:

1. TuningLines provides note label data via `get_note_labels()`
2. MainScene calls `text_backend.render_texts()` with the label data
3. The returned render objects are rendered to the background texture
4. The background texture is composited to the main screen

## Performance Considerations

- Font texture is created once and reused
- All possible note characters are pre-loaded to avoid texture update issues
- Pre-loading happens once on first render
- Minimal memory overhead with single texture

### Why Pre-loading is Necessary

Egui's font atlas uses partial texture updates when new glyphs are needed. However, partial updates only contain the new glyph data (e.g., 13x21 pixels). Since we only accept full texture replacements (to avoid corrupting the existing atlas), any characters not pre-loaded will appear as blank spaces. Pre-loading ensures all needed glyphs are in the initial texture.

## Technical Notes

- Text rendering uses transparency blending for proper overlay
- Material uses `Blend::TRANSPARENCY` and `WriteMask::COLOR`
- Font texture uses RGBA format with white RGB and alpha channel
- Color is applied through vertex colors in the mesh

## Code Organization

- `intonation-toy/presentation/egui_text_backend.rs`: Main text backend implementation
- `intonation-toy/presentation/tuning_lines.rs`: Provides note label data
- `intonation-toy/presentation/main_scene.rs`: Integrates text rendering
- No global state or complex dependencies