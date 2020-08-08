#version 450

layout(location = 0) in VertexData {
    vec2 coord;
} vertex;

layout(location = 0) out vec4 out_color;

void main() {
    vec2 mid = vec2(0.5, 0.5);
    if (distance(mid, vertex.coord) > 0.5) {
        discard;
    }
    out_color = vec4(1.0, 1.0, 1.0, 1.0);
}