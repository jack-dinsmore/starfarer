
#version 450

#extension GL_ARB_separate_shader_objects : enable

#define NUM_LIGHTS 2

layout (push_constant) uniform ObjectPushConstants {
    mat4 model;
    mat4 rotation;
} constants;
layout (binding = 0) uniform CameraData {
    mat4 view;
    mat4 proj;
    vec4 camera_pos;
} camera_ubo;

layout (location = 0) in vec3 inPosition;
layout (location = 1) in vec2 inUV;
layout (location = 2) in vec3 inNormal;
layout (location = 3) in vec4 inColor;
layout (location = 4) in vec3 inInfo;

layout (location = 0) out vec3 worldCoord;
layout (location = 1) out vec2 fragUV;
layout (location = 2) out vec3 fragNormal;
layout (location = 3) out vec4 fragColor;
layout (location = 4) out vec3 fragInfo;

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    worldCoord = (constants.model * vec4(inPosition, 1.0)).xyz; // Assumes that the model matrix does no scaling
    gl_Position = camera_ubo.proj * camera_ubo.view * vec4(worldCoord, 1.0);
    fragNormal = (constants.rotation * vec4(inNormal, 1.0)).xyz;
    fragColor = inColor;
    fragInfo = inInfo;
    fragUV = inUV;
}