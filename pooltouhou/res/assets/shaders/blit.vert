#version 450

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 coord;

layout(location = 0) out VertexData {
    vec2 coord;
} vertex;


void main() {

    vertex.coord = coord;

    gl_Position = vec4(pos, 0.0, 1.0);
}