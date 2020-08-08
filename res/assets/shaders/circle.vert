#version 450

layout(std140, set = 0, binding = 0) uniform Camera {
    uniform mat4 projection;
    uniform mat4 view;
    uniform mat4 model;
};


layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 coord;

layout(location = 0) out VertexData {
    vec2 coord;
} vertex;


void main() {

    vertex.coord = coord;

    gl_Position = projection * view * model * vec4(pos, 1.0);
}