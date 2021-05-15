#version 450

layout(location = 0) in VertexData {
    vec2 coord;
} vertex;

layout(set = 0, binding = 1) uniform texture2D t_diffuse;
layout(set = 0, binding = 2) uniform sampler s_diffuse;

layout(location = 0) out vec4 out_color;

void main() {
    float distanceX = 0.5 - vertex.coord.x;
    float distanceY = 0.5 - vertex.coord.y;
    if (distanceX * distanceX + distanceY * distanceY > 0.25) {
        discard;
    }
    out_color = vec4(1.0, 1.0, 1.0, 1.0);
}