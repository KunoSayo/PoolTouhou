#version 450


layout(location = 0) in vec2 coord;
layout(location = 1) in vec4 color;


layout(set = 1, binding = 0) uniform texture2D t;
layout(set = 1, binding = 1) uniform sampler s;

layout(location = 0) out vec4 out_color;

void main() {

    vec4 c = texture(sampler2D(t, s), coord);
    c = c * color;
    if (c.a <= 0.0) {
        discard;
    }
    out_color = c;
}