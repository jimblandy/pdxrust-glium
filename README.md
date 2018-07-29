# PDX Rust Meetup: Graphics In Rust

This is the example repository I used for my presentation, “Graphics in Rust” at
the June 2018 PDX Rust meetup. Unfortunately, I used a whiteboard instead of
slides, and the talk wasn't recorded (it was 2.5 hours long), so it's not really
available online. You should have been there! :D

If you like, however, you can try browsing the code in this repository. This is
my attempt to write the simplest possible program that still uses most of the
essential techniques that any graphical program will need.

This is not model Rust code, nor does it try to use OpenGL efficiently. Rather,
my goal is to present the API's behavior with as little interference from
abstraction and optimization as possible. So I don't use any of the pretty
vector arithmetic crates available (I recommend the `euclid` crate); it's all
functions that operate directly on `[f32; 3]` values. And you'll have to figure
out how to minimize CPU/GPU traffic and draw calls in the context of your app's
own needs; I just build vertex and index buffers from scratch and transfer them
whole to the GPU on every frame.

This repo contains several git branches, each showing the application at a
different stage of construction. The `master` branch is the fully constructed
demo, including all the code covered in the presentation. Other branches build
up the demo piece by piece, each introducing the use of one or two new OpenGL
features:

- `triangle`: Open a window and draw a single triangle in it. This is a pretty
  big step: it shows the mechanics of opening a window, building a vertex
  buffer, and using trivial vertex and fragment shaders.

- `spin`: Make the triangle spin around its axis. This computes a fresh vertex
  buffer on every frame, rather than using a uniform.

- `illuminated`: Light the triangle from the upper left. This supplies a normal
  vector along with each vertex, and then has the fragment shader use that
  normal to decide how the triangle is oriented relative to the light source.
  This then determines how brightly the triangle's surface is illuminated.

- `borders`: Add borders to the triangle. This demonstrates the use of line
  primitives.

- `three-triangles`: Expand the display to three triangles. This yields
  surprising results, since we haven't dealt with the triangles partially covering each other.

- `depth`: Add z-buffer and depth test, for occlusion. Now the triangles cover
  each other properly, even though the patterns of their intersection are pretty
  complicated.

- `perspective`: Introduce a second 'windmill', and apply a simple perspective
  transformation to make it seem further away.

- `texture`: Apply a texture to the vanes. We incorporate a PNG file into the
  executable, decompress it and upload it to the GPU, and then change the
  fragment shader to sample from it, applying the image's contents to the
  windmill vane. We have to expand each vertex with texture coordinates, so we
  can decide which fragment lands where in the texture.

Presently, `texture` is where the development ends, identical to `master`. It
might be nice to extend the demo to let the user control the camera position.

## Jank

Running this program on Linux (Fedora, GNOME Shell), I noticed that the rotation
is not completely smooth; there's a slight jank, occurring at regular intervals,
every second or so. I tried to debug this and didn't get anywhere—until I tried
simply opening up a terminal window and running the bash command:

    $ i=0; while true; do echo "hi $((i++))"; done

When I watch the numbers scrolling up the screen, they, too, jank periodically.
So I think this is an issue with the compositing done by the window manager, to
bring the demo's frame buffer onto the screen controlled by GNOME shell. I tried
the demo on a Mac, and it ran smoothly. So, at least for this, don't blame the
demo, or Glium, or Rust! :)
