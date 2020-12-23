#version 450

layout(location = 0) in vec2 vertexTextureCoordinates;
layout(location = 0) out vec4 fragmentColor;

layout(set = 0, binding = 0) uniform texture2D _texture;
layout(set = 0, binding = 1) uniform sampler _sampler;

void main() {
    fragmentColor = texture(sampler2D(_texture, _sampler), vertexTextureCoordinates);
}
