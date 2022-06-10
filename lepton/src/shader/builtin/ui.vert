
#version 450

#extension GL_ARB_separate_shader_objects : enable

layout (location = 0) in vec4 inPosition;
layout (location = 1) in vec2 inTexCoord;

layout (location = 1) out vec2 outTexCoord;

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    gl_Position = vec4(inPosition.x, inPosition.y, 0.0, 1.0);
    outTexCoord = inTexCoord;
}