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
}

////////////////
// SIMULATION //
////////////////

// TODO: Make this its own module?

#[derive(Debug, Clone)]
pub struct SimulationOptions {
    edge_length_force: f32,
}

impl Default for SimulationOptions {
    fn default() -> Self {
        Self {
            edge_length_force: 3.0,
        }
    }
}

impl Model {
    pub fn simulate(&mut self, options: &SimulationOptions, time_step: f32) {
        // List of forces, which we will accumulate as we go through the various forces
        let mut vert_forces = vec![Vec3::zero(); self.verts.len()];

        // Calculate forces from edges wanting to have length one
        for &(v_idx1, v_idx2) in &self.edges {
            let v1 = self.verts[v_idx1];
            let v2 = self.verts[v_idx2];

            let direction = v2 - v1;
            let length = direction.magnitude();
            let force_on_v2 = direction * (1.0 - length) * options.edge_length_force;
            let force_on_v1 = -force_on_v2;

            vert_forces[v_idx1] += force_on_v1;
            vert_forces[v_idx2] += force_on_v2;
        }

        // Update the vertex positions by a little bit in their movement directions
        for (force, v) in vert_forces.into_iter().zip_eq(&mut self.verts) {
            *v += force * time_step;
        }

        // TODO: recentre the model
    }
}

///////////////
// RENDERING //
///////////////

impl Model {
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
