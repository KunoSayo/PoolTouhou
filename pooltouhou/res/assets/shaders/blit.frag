#version 450

//layout(set = 0, binding = 0) uniform sampler2D t;
layout(set = 0, binding = 0) uniform texture2D t;
layout(set = 0, binding = 1) uniform sampler s;

layout(location = 0) in VertexData {
    vec2 coord;
} vertex;

layout(location = 0) out vec4 out_color;

void main() {
    out_color = texture(sampler2D(t, s), vertex.coord);
    //    out_color = texture(t, vertex.coord);
}