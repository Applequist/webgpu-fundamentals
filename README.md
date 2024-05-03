# WebGPU Fundamentals

IMPORTANT: The following are **verbatim** notes taken from the [WebGPU Fundamentals](https://webgpufundamentals.org)
websites for quick access. 
All credits are theirs.
Please head [there](https://webgpufundamentals.org) for the complete material.

## Fundamentals

## Inter-stage variables

Inter-stage variables come into play between a vertex shader and a fragment shader:
When a vertex shader outputs 3 positions, a triangle gets rasterized.
The vertex shader can output extra values **at each of those positions** and
by default, those values will be interpolated between the 3 points.

Inter-stage variables are most often used to interpolate texture coordinates
across a triangle, or normal directions.

IMPORTANT: the connection between the vertex shader and the fragment shader 
is **by index**. For inter-stage variables, they connect by `@location` index.

`@builtin` are **NOT** inter-stage variable, it's a... `@builtin`. 
It happens that `@builtin(position)` has a different meaning in a vertex shader 
vs in a fragment shader:
- in a vertex shader `@builtin(position)` is the output of the fragment shader in
  [clip space](https://www.w3.org/TR/webgpu/#coordinate-systems).
- in a fragment shader `@builtin(position)` is an input: it's the pixel coordinates
  in [framebuffer coordinates](https://www.w3.org/TR/webgpu/#coordinate-systems) 
  of the pixel the fragment shader is currently being asked to compute the color for.

TL/DR: 
- `@builtin(position)` in vertex shader and fragment shader are unrelated: they
are completely different variables.
- for inter-stage variables, all that matters are the `@location(n)` annotated variables/struct fields.

## Uniforms

Uniforms are kinda like global variables for your shaders: you set their values
before you execute the shader and they'll have those values for every iteration of the shader.
