# Egui Text Rendering Architecture

## Overview

The intonation-toy application uses egui for text rendering to display note labels on tuning lines. This document describes the architecture and implementation details of the text rendering system.

## Architecture

### EguiCompositeBackend

The text rendering is implemented through the `EguiCompositeBackend` struct, which provides a two-stage rendering process:

1. **Stage 1**: Render text using egui's off-screen rendering
2. **Stage 2**: Convert egui meshes to three-d renderables for integration with the main scene

### Key Components

```rust
pub struct EguiCompositeBackend {
    egui_ctx: egui::Context,           // Off-screen egui context
    queued_texts: Vec<QueuedText>,     // Text rendering queue
    texture_atlas: HashMap<egui::TextureId, Texture2DRef>, // Font texture cache
    pixels_per_point: f32,             // Display scaling factor
    glyph_cache_preloaded: bool,       // Pre-loading completion flag
}
```

## Text Rendering Pipeline

### 1. Text Queuing

Text is queued for rendering using the `queue_text()` method:

```rust
backend.queue_text("Db4", x, y, size, color);
```

### 2. Glyph Cache Pre-loading

To prevent partial texture atlas updates that would break rendering, all possible note characters are pre-loaded on first use:

```rust
let chars_to_preload = "CDEFGABb0123456789#";
```

This ensures the font atlas contains all characters needed for note names like "Db4", "C#5", etc.

### 3. Egui Rendering

The queued texts are rendered using egui's text system:

```rust
// Create text shapes
let galley = egui_ctx.fonts(|f| {
    f.layout_no_wrap(text, font_id, color)
});

let shape = egui::Shape::Text(egui::epaint::TextShape {
    pos, galley, color, ...
});
```

### 4. Tessellation

Egui converts the text shapes into renderable meshes:

```rust
let clipped_primitives = egui_ctx.tessellate(clipped_shapes, pixels_per_point);
```

### 5. Texture Atlas Management

The font texture atlas is managed with special handling for partial updates:

```rust
fn update_texture_atlas(&mut self, textures_delta: egui::TexturesDelta) {
    for (id, image_delta) in textures_delta.set {
        if image_delta.pos.is_some() {
            // Skip partial updates to prevent texture corruption
            continue;
        }
        // Process full texture updates only
        let texture = self.create_texture_from_image(context, &image_delta.image);
        self.texture_atlas.insert(id, texture);
    }
}
```

### 6. Mesh Conversion

Egui meshes are converted to three-d format for rendering:

```rust
fn convert_egui_mesh_to_three_d(
    mesh: &egui::epaint::Mesh,
    texture: Texture2DRef,
    viewport: Viewport,
) -> Option<Box<dyn Object>> {
    // Convert vertices, UVs, colors, and indices
    // Create GPU mesh and material
    // Return renderable object
}
```

## Critical Design Decisions

### Partial Update Handling

**Problem**: Egui's font atlas uses partial updates when new glyphs are needed. These partial updates only contain the new glyph data (e.g., 13x21 pixels), but replacing the entire texture atlas (2048x64) with this small data loses all existing glyphs.

**Solution**: 
1. **Pre-load all glyphs**: Load all possible note characters during initialization
2. **Skip partial updates**: Ignore texture updates with `pos: Some(...)` to preserve the complete font atlas
3. **Process full updates only**: Only accept complete texture replacements with `pos: None`

### Font Size Matching

**Problem**: Pre-loading must use the exact same font size as actual rendering, or glyphs won't be found in the atlas.

**Solution**: Pre-load at size 26.0 to match the note label rendering size exactly.

### Coordinate System Conversion

**Problem**: Egui uses top-left origin, three-d uses bottom-left origin.

**Solution**: Convert coordinates during mesh creation:
```rust
let converted_y = (viewport.height as f32 / pixels_per_point - egui_y) * pixels_per_point;
```

## Integration Points

### TuningLines Integration

The text rendering backend integrates with the tuning lines system:

```rust
// In TuningLines::render_note_labels()
for note in visible_notes {
    let note_name = midi_note_to_name(note);
    text_backend.queue_text(&note_name, x, y, size, color);
}

let text_objects = text_backend.create_text_models(context, viewport);
```

### Background Texture Rendering

Text objects are rendered directly into the background texture alongside tuning lines, ensuring proper compositing and performance.

## Performance Considerations

1. **Glyph Pre-loading**: One-time cost during initialization
2. **Texture Atlas Caching**: Font textures are reused across frames
3. **Mesh Reuse**: GPU meshes are created per frame but use efficient vertex buffers
4. **Minimal Character Set**: Only 18 unique characters pre-loaded

## Debugging Features

Debug logging is available with the `ATLAS_DEBUG` and `TEXT_DEBUG` prefixes:

- `ATLAS_DEBUG`: Texture atlas operations
- `TEXT_DEBUG`: Mesh conversion and rendering

## Future Improvements

1. **Dynamic Font Sizing**: Support multiple font sizes efficiently
2. **Text Caching**: Cache rendered text shapes across frames
3. **Better Error Handling**: Graceful degradation when font loading fails
4. **Font Customization**: Support for different fonts beyond Roboto

## Code Organization

- `intonation-toy/presentation/main_scene.rs`: Main implementation
- Text rendering is self-contained within the `EguiCompositeBackend` struct
- Integration points are minimal and well-defined
- No global state or complex dependencies

This architecture provides reliable, efficient text rendering while avoiding the pitfalls of egui's partial texture atlas updates.