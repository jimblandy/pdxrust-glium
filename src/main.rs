#[macro_use]
extern crate glium;

use glium::{Program, Surface, VertexBuffer};
use glium::glutin::{ContextBuilder, Event, EventsLoop, WindowBuilder, WindowEvent};
use glium::index::PrimitiveType;

use std::error::Error;
use std::f32::consts::PI;
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

/// Properties identifying windmill vane spinning about its axis of
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

impl Vane {
    /// Return the positions of this vane's three corners, with the vane
    /// rotated about its axis by `spin` radians.
    fn corners(&self) -> [[f32; 3]; 3] {
        let unit_towards_corner = mix_by_angle(&self.base_unit_i,
                                               &self.base_unit_j,
                                               self.spin);
        let base_midpt_to_corner = scale(&unit_towards_corner, self.base_radius);
        let corner1 = add(&self.base_midpt, &base_midpt_to_corner);
        let corner2 = subtract(&self.base_midpt, &base_midpt_to_corner);
        // Viewed from the front, our vertices must appear in clockwise order.
        [self.tip, corner1, corner2]
    }
}

#[derive(Clone, Copy, Debug)]
struct Vertex {
    position: [f32; 3]
}

implement_vertex!(Vertex, position);

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
        frame.clear_color(1.0, 1.0, 1.0, 1.0);

        let mut vertices = Vec::new();

        vane.spin = spin;
        vertices.extend(vane.corners().iter()
                        .map(|&position| Vertex { position }));
        assert_eq!(vertices.len(), 3);
        let vertex_buffer = VertexBuffer::new(&display, &vertices)?;

        frame.draw(&vertex_buffer, &glium::index::NoIndices(PrimitiveType::TrianglesList),
                   &vane_interiors_program,
                   &uniform! {}, &Default::default())?;
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
