struct OurStruct {
    color: vec4f,
    offset: vec2f,
};

struct OtherStruct {
    scale: vec2f,
};

@group(0) @binding(0) var<storage, read> ourStruct: OurStruct;
@group(0) @binding(1) var<storage, read> otherStruct: OtherStruct;

@vertex
fn vs(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    let x = f32(1 - i32(vertex_index)) * 0.5;
    let y = f32(i32(vertex_index & 1u) * 2 - 1) * 0.5;
    let position = vec2f(x, y);
    return vec4f(position * otherStruct.scale + ourStruct.offset, 0.0, 1.0);
}

@fragment
fn fs() -> @location(0) vec4f {
    return ourStruct.color;
}
