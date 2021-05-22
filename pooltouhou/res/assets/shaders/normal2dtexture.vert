#version 450

layout(std140, set = 0, binding = 0) uniform OutInfo {
    vec2 outSize;
};


layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 coord;
layout(location = 2) in vec4 color;

layout(location = 0) out VertexData {
    vec2 coord;
    vec4 color;
} vertex;


void main() {
    vertex.coord = coord;
    vertex.color = color;


    gl_Position = projection * view * vec4((pos.x / outSize.x) * 2 - 1, (pos.y / outSize.y) * 2 - 1, 0.0, 1.0);
}