use itertools::Itertools;
use three_d::*;

const EDGE_RADIUS: f32 = 0.05;
const VERT_RADIUS: f32 = 0.1;

#[derive(Debug, Clone, Default)]
pub struct Model {
    verts: Vec<Vec3>,
    edges: Vec<Edge>,

    symmetries: Vec<Mat3>,
    inverse_symmetries: Vec<Mat3>,
}

#[derive(Debug, Clone, Copy)]
struct Edge {
    vert_idx1: usize,
    vert_idx2: usize,
    symmetry_idx2: usize,
}

impl Model {
    // Creates an empty model
    pub fn polygon(n: usize, symmetry_factor: usize) -> Self {
        // Generate symmetries
        let symmetry_step_angle = Deg(360.0 / (symmetry_factor as f32));
        let symmetries = (0..symmetry_factor)
            .into_iter()
            .map(|i| Mat3::from_angle_z(symmetry_step_angle * (i as f32)))
            .collect_vec();

        assert!(n % symmetry_factor == 0);
        let verts_per_symmetry = n / symmetry_factor;

        // Generate vertices
        let mut verts = Vec::<Vec3>::new();
        for i in 0..verts_per_symmetry {
            let a = Deg(360.0 / (n as f32) * (i as f32 - 0.5));
            let (x, y) = a.sin_cos();
            verts.push(Vec3::new(y, x, 0.0) * 0.7);
        }

        // Generate edges within the part
        let mut edges = (0..verts_per_symmetry)
            .into_iter()
            .tuple_windows()
            .map(|(vert_idx1, vert_idx2)| Edge {
                vert_idx1,
                vert_idx2,
                symmetry_idx2: 0,
            })
            .collect_vec();
        // Edge which links to the next part
        edges.push(Edge {
            vert_idx1: verts_per_symmetry - 1,
            vert_idx2: 0,
            symmetry_idx2: 1,
        });

        Self {
            verts,
            edges,
            inverse_symmetries: symmetries.iter().map(|m| m.invert().unwrap()).collect_vec(),
            symmetries,
        }
    }

    pub fn add_polygon(&mut self, n: usize, verts: &[usize]) {
        // TODO: Calculate this properly
        let normal = Vec3::unit_y();

        let v1 = self.verts[verts[0]];
        let v2 = self.verts[verts[1]];
        let d = v2 - v1;

        let out = d.cross(normal).normalize();

        // Add verts
        let next_idx = self.verts.len();
        self.verts.push(v1 + out * d.magnitude());
        self.verts.push(v2 + out * d.magnitude());

        // Add edges
        self.edges.push(Edge {
            vert_idx1: verts[0],
            vert_idx2: next_idx + 0,
            symmetry_idx2: 0,
        });
        self.edges.push(Edge {
            vert_idx1: verts[1],
            vert_idx2: next_idx + 1,
            symmetry_idx2: 0,
        });
        self.edges.push(Edge {
            vert_idx1: next_idx + 0,
            vert_idx2: next_idx + 1,
            symmetry_idx2: 0,
        });

        dbg!(n, verts);
    }

    fn edge_vert_positions(&self, edge: &Edge) -> (Vec3, Vec3) {
        let v1 = self.verts[edge.vert_idx1];
        let v2 = self.symmetries[edge.symmetry_idx2] * self.verts[edge.vert_idx2];
        (v1, v2)
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
        for edge in &self.edges {
            let (v1, v2) = self.edge_vert_positions(edge);

            let direction = v2 - v1;
            let length = direction.magnitude();
            let force_on_v2 = direction * (1.0 - length) * options.edge_length_force;
            let force_on_v1 = -force_on_v2;

            vert_forces[edge.vert_idx1] += force_on_v1;
            vert_forces[edge.vert_idx2] +=
                self.inverse_symmetries[edge.symmetry_idx2] * force_on_v2;
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
        for symmetry in &self.symmetries {
            for edge in &self.edges {
                colors.push(crate::utils::egui_color_to_srgba(
                    crate::COLOR_THEME.sapphire,
                ));
                transformations.push(self.edge_transform(edge, symmetry));
            }
        }

        Instances {
            transformations,
            colors: Some(colors),
            ..Default::default()
        }
    }

    fn edge_transform(&self, edge: &Edge, symmetry: &Mat3) -> Mat4 {
        let (untransformed_p1, untransformed_p2) = self.edge_vert_positions(edge);
        let p1 = symmetry * untransformed_p1;
        let p2 = symmetry * untransformed_p2;

        Mat4::from_translation(p1)
            * Mat4::from(Quat::from_arc(
                vec3(1.0, 0.0, 0.0),
                (p2 - p1).normalize(),
                None,
            ))
            * Mat4::from_nonuniform_scale((p1 - p2).magnitude(), 1.0, 1.0)
    }

    pub fn vertex_mesh(&self, context: &three_d::Context) -> InstancedMesh {
        let mut sphere = CpuMesh::sphere(8);
        sphere.transform(Mat4::from_scale(VERT_RADIUS)).unwrap();

        let verts = self.vertex_instances();
        InstancedMesh::new(context, &verts, &sphere)
    }

    fn vertex_instances(&self) -> Instances {
        let color = crate::utils::egui_color_to_srgba(crate::COLOR_THEME.sapphire);

        let mut transformations = Vec::<Mat4>::new();
        for symmetry in &self.symmetries {
            for vert_pos in &self.verts {
                transformations.push(Mat4::from_translation(symmetry * vert_pos));
            }
        }

        Instances {
            transformations,
            colors: Some(vec![color; self.verts.len() * self.symmetries.len()]),
            ..Default::default()
        }
    }
}
