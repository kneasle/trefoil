use itertools::Itertools;
use three_d::*;

const EDGE_RADIUS: f32 = 0.05;
const VERT_RADIUS: f32 = 0.1;

#[derive(Debug, Clone, Default)]
pub struct Model {
    verts: Vec<Vec3>,
    edges: Vec<(usize, usize)>, // Indices into the 'verts' list
}

impl Model {
    // Creates an empty model
    pub fn polygon(n: usize) -> Self {
        let mut verts = Vec::<Vec3>::new();
        for i in 0..n {
            let a = Deg(360.0 / (n as f32) * (i as f32 + 0.5));
            let (x, y) = a.sin_cos();
            verts.push(Vec3::new(x, 0.0, y));
        }

        Self {
            verts,
            edges: (0..n).into_iter().circular_tuple_windows().collect_vec(),
        }
    }

    ///////////////
    // RENDERING //
    ///////////////

    pub fn edge_mesh(&self, context: &three_d::Context) -> InstancedMesh {
        let mut cylinder = CpuMesh::cylinder(10);
        cylinder
            .transform(Mat4::from_nonuniform_scale(1.0, EDGE_RADIUS, EDGE_RADIUS))
            .unwrap();

        let edges = self.edge_instances();
        InstancedMesh::new(context, &edges, &cylinder)
    }

    fn edge_instances(&self) -> Instances {
        let mut colors = Vec::new();
        let mut transformations = Vec::new();
        for &(v1, v2) in &self.edges {
            colors.push(crate::utils::egui_color_to_srgba(
                crate::COLOR_THEME.sapphire,
            ));
            transformations.push(edge_transform(self.verts[v1], self.verts[v2]));
        }

        Instances {
            transformations,
            colors: Some(colors),
            ..Default::default()
        }
    }

    pub fn vertex_mesh(&self, context: &three_d::Context) -> InstancedMesh {
        let mut sphere = CpuMesh::sphere(8);
        sphere.transform(Mat4::from_scale(VERT_RADIUS)).unwrap();

        let verts = self.vertex_instances();
        InstancedMesh::new(context, &verts, &sphere)
    }

    fn vertex_instances(&self) -> Instances {
        let color = crate::utils::egui_color_to_srgba(crate::COLOR_THEME.sapphire);
        Instances {
            transformations: self
                .verts
                .iter()
                .map(|&v| Mat4::from_translation(v))
                .collect_vec(),
            colors: Some(vec![color; self.verts.len()]),
            ..Default::default()
        }
    }
}

fn edge_transform(p1: Vec3, p2: Vec3) -> Mat4 {
    Mat4::from_translation(p1)
        * Mat4::from(Quat::from_arc(
            vec3(1.0, 0.0, 0.0),
            (p2 - p1).normalize(),
            None,
        ))
        * Mat4::from_nonuniform_scale((p1 - p2).magnitude(), 1.0, 1.0)
}
