#version 450

layout(std140, set = 0, binding = 0) uniform OutInfo {
    vec2 outSize;
};


layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 icoord;


layout(location = 0) out vec2 coord;

void main() {
    coord = icoord;

    gl_Position = vec4((pos.x / outSize.x) * 2 - 1, (pos.y / outSize.y) * 2 - 1, 0.0, 1.0);
}