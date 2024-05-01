# WebGPU Fundamentals

See [there](https://webgpufundamentals.org) for the material.

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

