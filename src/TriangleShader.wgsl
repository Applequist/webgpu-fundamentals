struct ColorOffset {
    color: vec4f,
    offset: vec2f,
};

struct Scales {
    scales: vec2f,
};

struct Vertex {
    position: vec2f,
}

@group(0) @binding(0) var<storage, read> color_offsets: array<ColorOffset>;
@group(0) @binding(1) var<storage, read> scales: array<Scales>;
@group(0) @binding(2) var<storage, read> pos: array<Vertex>;

struct VSOutput {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
}

@vertex
fn vs(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance_index: u32) -> VSOutput {
    let scales = scales[instance_index];
    let color_offset = color_offsets[instance_index];

    var vs_out: VSOutput;
    vs_out.position = vec4f(pos[vertex_index].position * scales.scales + color_offset.offset, 0.0, 1.0);
    vs_out.color = color_offset.color;

    return vs_out;
}

@fragment
fn fs(vs_out: VSOutput) -> @location(0) vec4f {
    return vs_out.color;
}
