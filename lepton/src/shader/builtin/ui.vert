
#version 450

#extension GL_ARB_separate_shader_objects : enable

layout (location = 0) in vec2 inPosition;
layout (location = 1) in vec2 inTexCoord;

layout (location = 1) out vec2 outTexCoord;

layout (push_constant) uniform UIPushConstants {
    float x;
    float y;
    float stretch_x;
    float stretch_y;
    vec4 color;
    float depth;
} constants;

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    gl_Position = vec4(
        inPosition.x * constants.stretch_x + constants.x,
        inPosition.y * constants.stretch_y + constants.y,
        constants.depth,
        1.0
    );
    outTexCoord = inTexCoord;
}