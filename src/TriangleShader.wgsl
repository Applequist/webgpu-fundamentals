struct Vertex {
    @builtin(position) position: vec4f,
};

@vertex
fn vs(@builtin(vertex_index) vertex_index: u32) -> Vertex {
    var vertex: Vertex;
    let x = f32(1 - i32(vertex_index)) * 0.5;
    let y = f32(i32(vertex_index & 1u) * 2 - 1) * 0.5;
    vertex.position = vec4f(x, y, 0.0, 1.0);
    return vertex;
}

@fragment
fn fs(v: Vertex) -> @location(0) vec4f {
    let red = vec4f(1, 0, 0, 1);
    let cyan = vec4f(0, 1, 1, 1);

    let grid = vec2u(v.position.xy) / 8;
    let checker = (grid.x + grid.y) % 2 == 1;
    return select(red, cyan, checker);
}

// Inter-stage variables connect between the vertex and fragment shader **by index**
// This would also work
//@fragment
//fn fs(@location(0) color: vec4f) -> @location(0) vec4f {
//    return color;
//}
