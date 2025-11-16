use itertools::Itertools;
use three_d::*;

#[derive(Debug, Clone)]
pub struct Model {}

impl Model {
    pub fn vertex_mesh(&self, context: &three_d::Context) -> InstancedMesh {
        let radius = 0.1;
        let mut sphere = CpuMesh::sphere(8);
        sphere.transform(Mat4::from_scale(radius)).unwrap();

        let verts = self.vertex_instances();
        InstancedMesh::new(context, &verts, &sphere)
    }

    fn vertex_instances(&self) -> Instances {
        let verts = vec![Vec3::zero(), Vec3::unit_z()];
        let color = crate::utils::egui_color_to_srgba(crate::COLOR_THEME.sapphire);
        Instances {
            transformations: verts
                .iter()
                .map(|&v| Mat4::from_translation(v))
                .collect_vec(),
            colors: Some(vec![color; verts.len()]),
            ..Default::default()
        }
    }
}
