
#version 450

#extension GL_ARB_separate_shader_objects : enable

#define NUM_LIGHTS 2

layout (binding = 0) uniform CameraData {
    mat4 model;
    mat4 view;
    mat4 proj;
    vec4 camera_pos;
    vec4 light_pos[NUM_LIGHTS];
    vec4 light_features[NUM_LIGHTS];
    uint num_lights;
} ubo;

layout (location = 0) in vec3 inPosition;
layout (location = 1) in vec3 inNormal;
layout (location = 2) in vec2 inTexCoord;

layout (location = 0) out vec3 fragNormal;
layout (location = 1) out vec2 fragTexCoord;
layout (location = 2) out vec3 worldCoord;

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    worldCoord = (ubo.model * vec4(inPosition, 1.0)).xyz; // Assumes that the model matrix does no scaling
    gl_Position = ubo.proj * ubo.view * vec4(worldCoord, 1.0);
    fragNormal = inNormal;
    fragTexCoord = inTexCoord;
}