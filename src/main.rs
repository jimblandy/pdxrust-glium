#[macro_use]
extern crate glium;

use glium::{Program, Surface, VertexBuffer};
use glium::draw_parameters::{BackfaceCullingMode, DrawParameters};
use glium::glutin::{ContextBuilder, Event, EventsLoop, WindowBuilder, WindowEvent};
use glium::index::PrimitiveType;

use std::error::Error;
use std::f32::consts::{PI, FRAC_PI_2};
use std::time::{Instant};

fn scale(a: &[f32; 3], scale: f32) -> [f32; 3] {
    [ a[0] * scale, a[1] * scale, a[2] * scale ]
}

fn negate(a: &[f32; 3]) -> [f32; 3] {
    scale(a, -1.0)
}

fn add(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn subtract(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    add(a, &negate(b))
}

/// Return the point `angle` radians around the origin-centered ellipse whose
/// major axis (center to zero-radians point) is `i` and whose minor axis
/// (center to π/2) is `j`.
fn mix_by_angle(i: &[f32; 3], j: &[f32; 3], angle: f32) -> [f32; 3] {
    add(&scale(i, angle.cos()),
        &scale(j, angle.sin()))
}

/// Properties identifying a windmill vane spinning about its axis of
/// symmetry in 3-space, with a distinguished front face.
struct Vane {
    /// Location of the vane's tip (the corner that lies on the axis of
    /// rotation).
    tip: [f32; 3],

    /// The midpoint of the vane's base (the side opposite the tip).
    base_midpt: [f32; 3],

    /// Half the length of base - the distance from the base's midpoint to each
    /// adjacent corner.
    base_radius: f32,

    /// Unit vector pointing from the midpoint of the base to the corner
    /// clockwise from the tip, in the unrotated state.
    base_unit_i: [f32; 3],

    /// Unit normal to the vane, pointing outwards from the front face,
    /// in the unrotated state.
    base_unit_j: [f32; 3],

    /// Rotation about the axis from the tip to base_midpt, in radians.
    spin: f32
}

enum Face { Front, Back }

impl Vane {
    /// Return the positions of the tree corners of the given face of this vane.
    fn corners(&self, face: Face) -> [[f32; 3]; 3] {
        let unit_towards_corner = mix_by_angle(&self.base_unit_i,
                                               &self.base_unit_j,
                                               self.spin);
        let base_midpt_to_corner = scale(&unit_towards_corner, self.base_radius);
        let corner1 = add(&self.base_midpt, &base_midpt_to_corner);
        let corner2 = subtract(&self.base_midpt, &base_midpt_to_corner);
        // Viewed from the front, each face's vertices must appear in clockwise order.
        match face {
            Face::Front => [self.tip, corner1, corner2],
            Face::Back  => [self.tip, corner2, corner1]
        }
    }
    /// Return a unit vector normal to the vane's given face.
    fn normal(&self, face: Face) -> [f32; 3] {
        let n = mix_by_angle(&self.base_unit_i, &self.base_unit_j,
                             self.spin + FRAC_PI_2);
        match face {
            Face::Front => n,
            Face::Back => negate(&n)
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3]
}

implement_vertex!(Vertex, position, normal);

fn main() -> Result<(), Box<Error>> {
    let mut events_loop = EventsLoop::new();
    let window = WindowBuilder::new()
        .with_dimensions(1000, 1000);
    let context = ContextBuilder::new()
        .with_vsync(true);
    let display = glium::Display::new(window, context, &events_loop)?;

    let vane_interiors_program =
        Program::from_source(&display,
                             &include_str!("vane.vert"),
                             &include_str!("interior.frag"),
                             None)?;
    let vane_interiors_draw_parameters =
        DrawParameters {
            backface_culling: BackfaceCullingMode::CullCounterClockwise,
            .. Default::default()
        };

    let mut vane = Vane {
        tip: [ 0.5, 0.0, 0.0 ],
        base_midpt: [ 0.0, 0.0, -0.5 ],
        base_radius: 0.5,
        base_unit_i: [ 0.0, -1.0, 0.0 ],
        base_unit_j: [ -0.707, 0.0, 0.707 ],
        spin: 0.0
    };

    let start_time = Instant::now();

    let mut window_open = true;
    while window_open {
        let frame_time = Instant::now() - start_time;

        let seconds = frame_time.as_secs() as f32 +
            (frame_time.subsec_nanos() as f32 * 1e-9);
        let spin = seconds * 0.125 * 2.0 * PI;

        let mut frame = display.draw();
        frame.clear_color(0.8, 0.8, 0.8, 1.0);

        let mut vertices = Vec::new();

        vane.spin = spin;
        let normal = vane.normal(Face::Front);
        vertices.extend(vane.corners(Face::Front).iter()
                        .map(|&position| Vertex { position, normal }));
        let backface_normal = vane.normal(Face::Back);
        vertices.extend(vane.corners(Face::Back).iter()
                        .map(|&position| Vertex { position, normal: backface_normal }));
        assert_eq!(vertices.len(), 6);
        let vertex_buffer = VertexBuffer::new(&display, &vertices)?;

        frame.draw(&vertex_buffer, &glium::index::NoIndices(PrimitiveType::TrianglesList),
                   &vane_interiors_program,
                   &uniform! {}, &vane_interiors_draw_parameters)?;
        frame.finish()?;

        events_loop.poll_events(|event| {
            match event {
                // Break from the main loop when the window is closed.
                Event::WindowEvent { event: WindowEvent::Closed, .. } => {
                    window_open = false;
                }
                _ => (),
            }
        });
    }
    Ok(())
}
