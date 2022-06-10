
#version 450

#extension GL_ARB_separate_shader_objects : enable

layout (binding = 3) uniform sampler2D texSampler;

layout (location = 1) in vec2 texCoord;

layout (location = 0) out vec4 outColor;

void main() {
    outColor = vec4(1.0, 0.0, 0.0, 0.5); //texture(texSampler, texCoord);
}

