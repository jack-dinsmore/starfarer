
#version 450

#extension GL_ARB_separate_shader_objects : enable

#define NUM_LIGHTS 2

layout (binding = 1) uniform CameraData {
    mat4 view;
    mat4 proj;
    vec4 camera_pos;
} camera_ubo;
layout (binding = 2) uniform LightsData {
    vec4 light_pos[NUM_LIGHTS];
    vec4 light_features[NUM_LIGHTS];
    uint num_lights;
} lights_ubo;

layout (binding = 0) uniform sampler2D texSampler;


layout (location = 0) in vec3 worldCoord;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec4 fragColor;
layout (location = 3) in vec3 fragInfo;

layout (location = 0) out vec4 outColor;

void main() {
    const vec3 camera_pos = normalize(camera_ubo.camera_pos.xyz - worldCoord);

    float illumination = 0;
    for (uint index = 0; index < lights_ubo.num_lights; index++) {
        const vec3 light_source = normalize(lights_ubo.light_pos[index].xyz - worldCoord);
        const vec3 reflection = normalize(2 * dot(light_source, normal) * normal - light_source);
        const float normal_dot = dot(light_source, normal);
        if (normal_dot < 0.0) {
            continue;
        }
        illumination += lights_ubo.light_features[index].w * (
            normal_dot + 
            fragInfo.x * pow(max(dot(reflection, camera_pos), 0), fragInfo.y));
    }

    outColor = vec4(fragColor.xyz * illumination, fragColor.a);
}