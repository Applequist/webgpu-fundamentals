struct Vertex {
    @location(0) position: vec2f,
    @location(1) color: vec4f,
    @location(2) offset: vec2f,
    @location(3) scales: vec2f,
    @location(4) per_vertex_color: vec3f,
}

struct VSOutput {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
}

@vertex
fn vs(vertex: Vertex) -> VSOutput {
    var vs_out: VSOutput;
    vs_out.position = vec4f(vertex.position * vertex.scales + vertex.offset, 0.0, 1.0);
    vs_out.color = vertex.color * vec4f(vertex.per_vertex_color, 1.);

    return vs_out;
}

@fragment
fn fs(vs_out: VSOutput) -> @location(0) vec4f {
    return vs_out.color;
}
