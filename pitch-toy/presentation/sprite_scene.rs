//! Sprite Scene - Placeholder WebGL scene with red sprites
//! 
//! This is a PLACEHOLDER implementation used for development and testing.
//! It demonstrates basic 3D rendering with the three-d library.
//! 
//! TODO: Replace this with actual pitch visualization components when
//! the presentation layer is fully implemented.
//! 
//! This file can be safely deleted once proper visualization is implemented.

use three_d::{Context, Viewport, Camera, Axes, ColorMaterial, Sprites, AmbientLight, Srgba, ClearState, Gm, RenderTarget, vec3, degrees, InnerSpace};

/// PLACEHOLDER WebGL scene with red sprites and camera
/// 
/// This is a temporary implementation used to demonstrate basic 3D rendering
/// capabilities while the actual pitch visualization is being developed.
/// 
/// # Purpose
/// - Provides a visual confirmation that the 3D rendering pipeline works
/// - Serves as a basic example of three-d library usage
/// - Acts as a placeholder until proper pitch/volume visualization is implemented
/// 
/// # Removal
/// This struct and file should be deleted once the presentation layer
/// has proper pitch visualization components.
pub struct SpriteScene {
    camera: Camera,
    axes: Axes,
    material: ColorMaterial,
    billboards: Sprites,
    sprites_up: Sprites,
    sprites: Sprites,
    ambient: AmbientLight,
}

impl SpriteScene {
    /// Create new placeholder sprite scene
    /// 
    /// Sets up a basic 3D scene with red sprites positioned at various locations
    /// to demonstrate 3D rendering capabilities.
    pub fn new(context: &Context, viewport: Viewport) -> Self {
        let camera = Camera::new_perspective(
            viewport,
            vec3(0.0, 15.0, 15.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            degrees(60.0),
            0.1,
            1000.0,
        );

        let axes = Axes::new(context, 0.1, 1.0);

        let material = ColorMaterial {
            color: Srgba::new(255, 100, 100, 255), // Red color
            ..Default::default()
        };

        let billboards = Sprites::new(
            context,
            &[
                vec3(-20.0, 0.0, -5.0),
                vec3(-15.0, 0.0, -10.0),
                vec3(-10.0, 0.0, -5.0),
            ],
            None,
        );

        let sprites_up = Sprites::new(
            context,
            &[
                vec3(5.0, 0.0, -5.0),
                vec3(0.0, 0.0, -10.0),
                vec3(-5.0, 0.0, -5.0),
            ],
            Some(vec3(0.0, 1.0, 0.0)),
        );

        let sprites = Sprites::new(
            context,
            &[
                vec3(20.0, 0.0, -5.0),
                vec3(15.0, 0.0, -10.0),
                vec3(10.0, 0.0, -5.0),
            ],
            Some(vec3(1.0, 1.0, 0.0).normalize()),
        );

        let ambient = AmbientLight::new(context, 1.0, Srgba::WHITE);

        Self {
            camera,
            axes,
            material,
            billboards,
            sprites_up,
            sprites,
            ambient,
        }
    }

    /// Update camera viewport for window resize
    pub fn update_viewport(&mut self, viewport: Viewport) {
        self.camera.set_viewport(viewport);
    }

    /// Render the placeholder scene
    /// 
    /// Renders the red sprites and coordinate axes to the screen.
    /// This is purely for demonstration purposes.
    pub fn render(&self, screen: &mut RenderTarget) {
        screen
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
            .render(
                &self.camera,
                self.axes.into_iter()
                    .chain(&Gm {
                        geometry: &self.billboards,
                        material: &self.material,
                    })
                    .chain(&Gm {
                        geometry: &self.sprites_up,
                        material: &self.material,
                    })
                    .chain(&Gm {
                        geometry: &self.sprites,
                        material: &self.material,
                    }),
                &[&self.ambient],
            );
    }
}