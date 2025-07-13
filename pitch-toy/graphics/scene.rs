// Sprite Scene - WebGL scene with red sprites using three-d

use three_d::*;

/// WebGL scene with red sprites and camera
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
    /// Create new sprite scene
    pub fn new(context: &Context, viewport: Viewport) -> Self {
        let mut camera = Camera::new_perspective(
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

    /// Update camera viewport
    pub fn update_viewport(&mut self, viewport: Viewport) {
        self.camera.set_viewport(viewport);
    }

    /// Render the scene
    pub fn render(&self, screen: &Screen) {
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