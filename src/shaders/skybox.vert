
#version 450

#extension GL_ARB_separate_shader_objects : enable
layout (binding = 0) uniform CameraData {
    mat4 view;
    mat4 proj;
    vec4 camera_pos;
} camera_ubo;

layout (location = 0) in vec3 inPosition;
layout (location = 1) in vec2 inTexCoord;

layout (location = 0) out vec2 fragTexCoord;
layout (location = 1) out vec3 pointCoord;

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    gl_Position = camera_ubo.proj * camera_ubo.view * (vec4(inPosition, 1.0) + camera_ubo.camera_pos);
    fragTexCoord = inTexCoord;
    pointCoord = inPosition;
}