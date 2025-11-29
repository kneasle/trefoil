use itertools::Itertools;
use three_d::{egui::Vec2, *};

const EDGE_RADIUS: f32 = 0.1;
const VERT_RADIUS: f32 = 0.15;

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
        let radius = 0.5 / Deg(180.0 / n as f32).sin();
        let mut verts = Vec::<Vec3>::new();
        for i in 0..verts_per_symmetry {
            let a = Deg(360.0 / (n as f32) * (i as f32 - 0.5));
            let (x, y) = a.sin_cos();
            verts.push(Vec3::new(y, x, 0.0) * radius);
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

    pub fn add_polygon(&mut self, n: usize, verts: &[usize], override_normal: Option<Vec3>) {
        // Calculate the normal to the current chain of vertices
        assert!(verts.len() >= 2);
        let normal;
        if let Some(norm) = override_normal {
            normal = norm.normalize();
        } else {
            assert!(verts.len() >= 3);
            let mut total_normal = Vec3::zero();
            for (i1, i2, i3) in verts.iter().tuple_windows() {
                let v1 = self.verts[*i1];
                let v2 = self.verts[*i2];
                let v3 = self.verts[*i3];
                total_normal += (v3 - v2).cross(v2 - v1);
            }
            normal = total_normal.normalize();
        }

        let first_vert_idx = *verts.first().unwrap();
        let last_vert_idx = *verts.last().unwrap();

        // We want to distribute `num_new_verts` as a continuation of the two ends of `verts`
        let num_new_verts = n - verts.len();
        if num_new_verts == 0 {
            // TODO: Just join the existing two verts
            todo!();
        }

        // Get two perpendicular axes to define the plane in which we will build our polygon
        let v1 = self.verts[first_vert_idx];
        let v2 = self.verts[last_vert_idx];
        let d = v2 - v1;
        let out = d.cross(normal);

        // Calculate how far away from line (v1-v2) the centre of the new polygon ought to be,
        // as a multiple of `out`.
        let half_radial_angle = Deg(180.0) - Deg(180.0) / (n as f32) * (num_new_verts as f32 + 1.0);
        let polygon_centre_dist = 0.5 / half_radial_angle.tan();

        // Add verts
        let idx_of_first_vert = self.verts.len();
        let angle_step = Deg(360.0) / (n as f32);
        let polygon_radius = Vec2::new(0.5, polygon_centre_dist).length();
        for v_idx in 0..num_new_verts {
            // Interpreted as angle from the -y axis
            let angle = angle_step * (v_idx as f32 + 1.0) + half_radial_angle;
            let x = 0.5 - angle.sin() * polygon_radius;
            let y = polygon_centre_dist - angle.cos() * polygon_radius;
            self.verts.push(v1 + d * x + out * y);
        }

        // Add edges between the new verts
        for (i1, i2) in (0..num_new_verts).into_iter().tuple_windows() {
            self.edges.push(Edge {
                vert_idx1: idx_of_first_vert + i1,
                vert_idx2: idx_of_first_vert + i2,
                symmetry_idx2: 0,
            });
        }
        // Link the ends of the new chains of verts to existing verts
        self.edges.push(Edge {
            vert_idx1: first_vert_idx,
            vert_idx2: idx_of_first_vert,
            symmetry_idx2: 0,
        });
        self.edges.push(Edge {
            vert_idx1: last_vert_idx,
            vert_idx2: idx_of_first_vert + num_new_verts - 1,
            symmetry_idx2: 0,
        });
    }

    pub fn ensure_all_verts_have_three_edges(&mut self) {
        for (vert_idx, edge_directions) in
            self.get_edge_directions_per_vert().into_iter().enumerate()
        {
            match edge_directions.as_slice() {
                [dir1, dir2] => {
                    let new_vert_idx = self.verts.len();
                    self.verts
                        .push(self.verts[vert_idx] - (dir1 + dir2).normalize());
                    self.edges.push(Edge {
                        vert_idx1: vert_idx,
                        vert_idx2: new_vert_idx,
                        symmetry_idx2: 0, // Both exist in the same symmetrical section
                    });
                }
                _ => {}
            }
        }
    }

    fn get_edge_directions_per_vert(&mut self) -> Vec<Vec<Vec3>> {
        let mut edges_per_vert = vec![Vec::<Vec3>::new(); self.verts.len()];
        for edge in &self.edges {
            let (v1, v2) = self.edge_vert_positions(edge);
            let d1 = v2 - v1;
            let d2 = self.inverse_symmetries[edge.symmetry_idx2] * (v1 - v2);
            edges_per_vert[edge.vert_idx1].push(d1);
            edges_per_vert[edge.vert_idx2].push(d2);
        }
        edges_per_vert
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
    pub edge_length_force: f32,
    pub vertex_angle_force: f32,
}

impl Default for SimulationOptions {
    fn default() -> Self {
        Self {
            edge_length_force: 10.0,
            vertex_angle_force: 2.0,
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

        // Calculate forces from vertices wanting to have all their edges at 120
        //
        // TODO: Cache the adjacency
        let mut edges_per_vert = vec![Vec::<Vec3>::new(); self.verts.len()];
        for edge in &self.edges {
            let (v1, v2) = self.edge_vert_positions(edge);
            let d1 = v2 - v1;
            let d2 = self.inverse_symmetries[edge.symmetry_idx2] * (v1 - v2);
            edges_per_vert[edge.vert_idx1].push(d1);
            edges_per_vert[edge.vert_idx2].push(d2);
        }
        for (vert_idx, edge_directions) in edges_per_vert.iter().enumerate() {
            if edge_directions.len() == 3 {
                let mut total_edge_dir = Vec3::zero();
                for dir in edge_directions {
                    total_edge_dir += *dir;
                }
                vert_forces[vert_idx] += total_edge_dir * options.vertex_angle_force;
            }
        }

        // Update the vertex positions by a little bit in their movement directions
        for (force, v) in vert_forces.into_iter().zip_eq(&mut self.verts) {
            *v += force * time_step;
        }
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
                colors.push(crate::utils::egui_color_to_srgba(crate::COLOR_THEME.red));
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
        let color = crate::utils::egui_color_to_srgba(crate::COLOR_THEME.red);

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

    pub fn vertex_texts(&self) -> Vec<(String, Vec3)> {
        let mut strings = Vec::<(String, Vec3)>::new();
        for (symm_idx, symmetry) in self.symmetries.iter().enumerate() {
            for (vert_idx, vert_pos) in self.verts.iter().enumerate() {
                strings.push((format!("{vert_idx}:{symm_idx}"), symmetry * vert_pos));
            }
        }
        strings
    }
}
