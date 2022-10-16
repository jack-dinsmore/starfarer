
#version 450

#extension GL_ARB_separate_shader_objects : enable

layout (push_constant) uniform SkyboxPushConstants {
    vec4 planet_pos;
    vec4 sun_pos;
    vec4 planet_info; // Zero if allowed, radius, scale height, min_alpha
} constants;

layout (set=0, binding = 0) uniform CameraData {
    mat4 view;
    mat4 proj;
    vec4 camera_pos;
} camera_ubo;
layout (set=0, binding = 1) uniform sampler2D texSampler;

layout (set=1, binding = 2) uniform sampler2D skySampler;

layout (location = 0) in vec2 texCoord;
layout (location = 1) in vec3 pointCoord;

layout (location = 0) out vec4 outColor;

void main() {
    if (constants.planet_info.x != 0.0) {
        float alpha = exp(-
        (length(constants.planet_pos - camera_ubo.camera_pos) - constants.planet_info.y)
         / constants.planet_info.z) * constants.planet_info.w;

        vec3 person_pos = normalize(camera_ubo.camera_pos.xyz - constants.planet_pos.xyz);
        vec3 sun_pos = normalize(constants.sun_pos.xyz - constants.planet_pos.xyz);
        vec3 to_sun = normalize(constants.sun_pos.xyz - camera_ubo.camera_pos.xyz);
        vec2 skyCoord = vec2(dot(person_pos, sun_pos), dot(pointCoord, to_sun));

        outColor = (1.0 - alpha) * texture(skySampler, texCoord) + alpha * texture(texSampler, skyCoord);
    } else {
        outColor = texture(skySampler, texCoord);
    }
}