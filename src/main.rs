#[macro_use]
extern crate glium;
extern crate image;

use glium::{Display, IndexBuffer, Program, Surface, VertexBuffer};
use glium::draw_parameters::{BackfaceCullingMode, DrawParameters};
use glium::glutin::{ContextBuilder, Event, EventsLoop, WindowBuilder, WindowEvent};
use glium::index::PrimitiveType;
use glium::texture::{Texture2d, RawImage2d};
use glium::uniforms::SamplerWrapFunction;

use std::error::Error;
use std::f32::consts::{PI, FRAC_PI_2};
use std::io::Cursor;
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

fn midpoint(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    scale(&add(a, b), 0.5)
}

fn length(a: &[f32; 3]) -> f32 {
    f32::sqrt(a[0] * a[0] + a[1] * a[1] + a[2] * a[2])
}

fn normalize(a: &[f32; 3]) -> [f32; 3] {
    let inverse_length = 1.0 / length(a);
    assert!(!inverse_length.is_infinite());
    scale(a, inverse_length)
}

/// Return the point `angle` radians around the origin-centered ellipse whose
/// major axis (center to zero-radians point) is `i` and whose minor axis
/// (center to π/2) is `j`.
fn mix_by_angle(i: &[f32; 3], j: &[f32; 3], angle: f32) -> [f32; 3] {
    add(&scale(i, angle.cos()),
        &scale(j, angle.sin()))
}

/// Return a unit vector in the XY plane that is rotated `angle` radians
/// counter-clockwise from the X axis.
fn unit_at_angle(angle: f32) -> [f32; 3] {
    [ angle.cos(), angle.sin(), 0.0 ]
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

    /// Return the positions of each corner of the given `face` in the vane's
    /// texture space.
    fn texture_corners(&self, face: Face) -> [[f32; 2]; 3] {
        match face {
            Face::Front => [[ 0.50, 0.05 ],
                            [ 0.95, 0.95 ],
                            [ 0.05, 0.95 ]],
            Face::Back  => [[ 0.50, 0.05 ],
                            [ 0.05, 0.95 ],
                            [ 0.95, 0.95 ]]
        }
    }
}

static VANE_TEXTURE : &'static [u8] = include_bytes!("rainbow-vane-small.png");

fn build_vane_texture(display: &Display) -> Result<Texture2d, Box<Error>>
{
    let image = image::load(Cursor::new(VANE_TEXTURE), image::PNG)?.to_rgba();
    let (width, height) = image.dimensions();
    let raw_image = RawImage2d::from_raw_rgba(image.into_raw(), (width, height));
    Ok(Texture2d::new(display, raw_image)?)
}

#[derive(Clone, Copy, Debug)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    texture: [f32; 2]
}

implement_vertex!(Vertex, position, normal, texture);

fn main() -> Result<(), Box<Error>> {
    let mut events_loop = EventsLoop::new();
    let window = WindowBuilder::new()
        .with_dimensions(1000, 1000);
    let context = ContextBuilder::new()
        .with_vsync(true)
        .with_depth_buffer(24);
    let display = glium::Display::new(window, context, &events_loop)?;

    let vane_interiors_program =
        Program::from_source(&display,
                             &include_str!("vane.vert"),
                             &include_str!("interior.frag"),
                             None)?;
    let vane_interiors_draw_parameters =
        DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfMore,
                write: true,
                .. Default::default()
            },
            backface_culling: BackfaceCullingMode::CullCounterClockwise,
            .. Default::default()
        };

    let vane_borders_program =
        Program::from_source(&display,
                             &include_str!("vane.vert"),
                             &include_str!("borders.frag"),
                             None)?;
    let vane_borders_draw_parameters =
        DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfMore,
                write: true,
                .. Default::default()
            },
            line_width: Some(2.0),
            .. Default::default()
        };

    let vane_texture = build_vane_texture(&display)?;

    fn vane(pt: &[f32; 3], angle: f32) -> Vane {
        let inner_radius = 0.25;
        let outer_radius = 0.5;

        let unit_tip = unit_at_angle(angle);
        let unit1    = unit_at_angle(angle + PI * 7.0 / 6.0);
        let unit2    = unit_at_angle(angle + PI * 5.0 / 6.0);

        let tip = scale(&unit_tip, inner_radius);
        let corner1 = scale(&unit1, outer_radius);
        let corner2 = scale(&unit2, outer_radius);
        let base_midpt = midpoint(&corner1, &corner2);
        let base_midpt_to_corner1 = subtract(&corner1, &base_midpt);

        Vane {
            tip: add(pt, &tip),
            base_midpt: add(pt, &base_midpt),
            base_radius: length(&base_midpt_to_corner1),
            base_unit_i: normalize(&base_midpt_to_corner1),
            base_unit_j: [ 0.0, 0.0, 1.0 ],
            spin: 0.0
        }
    }

    let mut vanes = [
        vane(&[0.0, 0.0, 0.0], 0.0),
        vane(&[0.0, 0.0, 0.0], PI * 2.0 / 3.0),
        vane(&[0.0, 0.0, 0.0], PI * 4.0 / 3.0),

        vane(&[1.0, 1.0, -2.5], 0.0),
        vane(&[1.0, 1.0, -2.5], PI * 2.0 / 3.0),
        vane(&[1.0, 1.0, -2.5], PI * 4.0 / 3.0),
    ];

    let start_time = Instant::now();

    let mut window_open = true;
    while window_open {
        let frame_time = Instant::now() - start_time;

        let seconds = frame_time.as_secs() as f32 +
            (frame_time.subsec_nanos() as f32 * 1e-9);
        let spin = seconds * 0.125 * 2.0 * PI;

        let mut frame = display.draw();
        frame.clear_color_and_depth((0.8, 0.8, 0.8, 1.0), 0.0);

        let mut vertices = Vec::new();

        for (i, vane) in vanes.iter_mut().enumerate() {
            let spin = spin * (if i >= 3 { 0.5 } else { 1.0 });
            vane.spin = spin + i as f32;
        }

        // Put the front faces first; we'll re-use them as vertices for the
        // border lines.
        for vane in &vanes {
            let normal = vane.normal(Face::Front);
            vertices.extend(vane.corners(Face::Front).iter()
                            .zip(vane.texture_corners(Face::Front).iter())
                            .map(|(&position, &texture)|
                                 Vertex { position, normal, texture }));
        }

        for vane in &vanes {
            let normal = vane.normal(Face::Back);
            vertices.extend(vane.corners(Face::Back).iter()
                            .zip(vane.texture_corners(Face::Back).iter())
                            .map(|(&position, &texture)|
                                 Vertex { position, normal, texture }));
        }

        let vertex_buffer = VertexBuffer::new(&display, &vertices)?;
        frame.draw(&vertex_buffer, &glium::index::NoIndices(PrimitiveType::TrianglesList),
                   &vane_interiors_program,
                   &uniform! {
                       vane_texture: vane_texture
                           .sampled()
                           .wrap_function(SamplerWrapFunction::Clamp)
                   },
                   &vane_interiors_draw_parameters)?;

        // Reuse just the front faces' vertices for the borders.
        let border_vertex_buffer = VertexBuffer::new(&display, &vertices[0..18])?;
        let indices: Vec<u16> = (0..6)
            .flat_map(|n| {
                let i = n * 3;
                vec![ i, i+1,
                      i+1, i+2,
                      i+2, i ]
            })
            .collect();
        let border_index_buffer = IndexBuffer::new(&display, PrimitiveType::LinesList,
                                                   &indices)?;
        frame.draw(&border_vertex_buffer, &border_index_buffer,
                   &vane_borders_program,
                   &uniform! {}, &vane_borders_draw_parameters)?;

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
