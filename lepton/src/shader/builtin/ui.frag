
#version 450

#extension GL_ARB_separate_shader_objects : enable

layout (binding = 0) uniform sampler2D texSampler;
layout (push_constant) uniform UIPushConstants {
    float x;
    float y;
    float stretch_x;
    float stretch_y;
    vec4 color;
} constants;

layout (location = 1) in vec2 texCoord;

layout (location = 0) out vec4 outColor;

void main() {
    outColor = texture(texSampler, texCoord).r * constants.color;
}