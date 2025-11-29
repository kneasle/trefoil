use std::ops::Deref;

use three_d::*;

/// The 3D viewport used to display a model
pub(crate) struct Viewport {
    context: Context,
    text_generator: TextGenerator<'static>,

    camera: Camera,
    control: OrbitControl,

    wireframe_material: PhysicalMaterial,
    text_material: ColorMaterial,
}

impl Viewport {
    pub fn new(context: &Context, viewport: three_d::Viewport) -> Self {
        // Camera
        let target = vec3(0.0f32, 0.0, 0.0);
        let scene_radius = 6.0f32;
        let camera = Camera::new_perspective(
            viewport,
            target + scene_radius * vec3(0.0, 0.0, 1.0),
            target,
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            0.1,
            1000.0,
        );
        let control = OrbitControl::new(camera.target(), 0.1 * scene_radius, 100.0 * scene_radius);

        let mut wireframe_material = PhysicalMaterial::new_opaque(
            context,
            &CpuMaterial {
                albedo: Srgba::WHITE,
                roughness: 0.7,
                metallic: 0.0,
                ..Default::default()
            },
        );
        wireframe_material.render_states.cull = Cull::Back;

        let mut text_material = ColorMaterial::new_opaque(
            context,
            &CpuMaterial {
                albedo: Srgba::WHITE,
                ..Default::default()
            },
        );
        text_material.render_states.cull = Cull::None;

        let text_generator =
            TextGenerator::new(include_bytes!("FiraMono-Medium.ttf"), 0, 30.0).unwrap();

        Self {
            context: context.clone(),
            text_generator,

            camera,
            control,

            wireframe_material,
            text_material,
        }
    }

    pub fn update(&mut self, frame_input: &mut FrameInput, viewport: three_d::Viewport) -> bool {
        let mut redraw = frame_input.first_frame;
        redraw |= self.camera.set_viewport(viewport);
        redraw |= self
            .control
            .handle_events(&mut self.camera, &mut frame_input.events);
        redraw
    }

    pub fn render(&mut self, model: &crate::model::Model, target: &RenderTarget) {
        // Lights
        let ambient = AmbientLight::new(&self.context, 0.7, Srgba::WHITE);
        let directional0 =
            DirectionalLight::new(&self.context, 2.0, Srgba::WHITE, vec3(-1.0, -1.0, -1.0));
        let directional1 =
            DirectionalLight::new(&self.context, 2.0, Srgba::WHITE, vec3(1.0, 1.0, 1.0));
        let lights = [&ambient as &dyn Light, &directional0, &directional1];

        // Create an example mesh for illustration
        let radius = 0.1;
        let mut sphere = CpuMesh::sphere(8);
        sphere.transform(Mat4::from_scale(radius)).unwrap();

        // Meshes
        // TODO: Add some caching to not send these to the GPU every frame
        let mut meshes: Vec<Box<dyn Object>> = Vec::new();

        // Text
        meshes.push(Box::new(Gm::new(
            Mesh::new(
                &self.context,
                &self
                    .text_generator
                    .generate("Hello!", TextLayoutOptions::default()),
            ),
            &self.text_material,
        )));

        meshes.push(Box::new(Gm::new(
            model.edge_mesh(&self.context),
            &self.wireframe_material,
        )));
        meshes.push(Box::new(Gm::new(
            model.vertex_mesh(&self.context),
            &self.wireframe_material,
        )));

        target.render(&self.camera, meshes.iter().map(Deref::deref), &lights);
    }
}
