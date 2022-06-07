
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

layout (binding = 1) uniform sampler2D texSampler;

layout (location = 0) in vec3 normal;
layout (location = 1) in vec2 texCoord;
layout (location = 2) in vec3 worldCoord;

layout (location = 0) out vec4 outColor;

void main() {
    const vec3 camera_pos = normalize(ubo.camera_pos.xyz - worldCoord);

    float illumination = 0;
    for (uint index = 0; index < ubo.num_lights; index++) {
        const vec3 light_source = normalize(ubo.light_pos[index].xyz - worldCoord);
        const vec3 reflection = normalize(2 * dot(light_source, normal) * normal - light_source);
        illumination +=
            ubo.light_features[index].x * max(dot(light_source, normal), 0) + 
            ubo.light_features[index].y * pow(max(dot(reflection, camera_pos), 0), ubo.light_features[index].z);
    }

    outColor = texture(texSampler, texCoord) * illumination;
}

