struct OurStruct {
    color: vec4f,
    scale: vec2f,
    offset: vec2f,
};

@group(0) @binding(0) var<uniform> ourStruct: OurStruct;

@vertex
fn vs(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    let x = f32(1 - i32(vertex_index)) * 0.5;
    let y = f32(i32(vertex_index & 1u) * 2 - 1) * 0.5;
    let position = vec2f(x, y);
    return vec4f(position * ourStruct.scale + ourStruct.offset, 0.0, 1.0);
}

@fragment
fn fs() -> @location(0) vec4f {
    return ourStruct.color;
}
