// shader.vert
#version 450

const vec2 verteces[6] = vec2[6](
    vec2(-0.5, -0.5),
    vec2(0.5, -0.5),
    vec2(0.5, 0.5),
    vec2(0.5, 0.5),
    vec2(-0.5, 0.5),
    vec2(-0.5, -0.5)
);

void main() {
    gl_Position = vec4(verteces[gl_VertexIndex], 0.0, 1.0);
}
