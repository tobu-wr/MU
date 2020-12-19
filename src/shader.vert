#version 450

const vec2 positions[6] = vec2[6](
    vec2(-1, -1),
    vec2(1, -1),
    vec2(1, 1),
    vec2(1, 1),
    vec2(-1, 1),
    vec2(-1, -1)
);

const vec2 textureCoordinates[6] = vec2[6](
    vec2(0, 1),
    vec2(1, 1),
    vec2(1, 0),
    vec2(1, 0),
    vec2(0, 0),
    vec2(0, 1)
);

layout(location = 0) out vec2 vertexTextureCoordinates;

void main() {
    gl_Position = vec4(positions[gl_VertexIndex], 0, 1);
    vertexTextureCoordinates = textureCoordinates[gl_VertexIndex];
}
