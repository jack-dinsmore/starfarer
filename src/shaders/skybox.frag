
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
layout (set=0, binding = 1) uniform sampler2D skycolorSampler;
layout (set=1, binding = 2) uniform sampler2D skyboxSampler;

layout (location = 0) in vec2 texCoord;
layout (location = 1) in vec3 pointCoord;

layout (location = 0) out vec4 outColor;

void main() {
    if (constants.planet_info.x != 0.0) {
        float alpha = exp(-
        (length(constants.planet_pos - camera_ubo.camera_pos) - constants.planet_info.y)
         / constants.planet_info.z) * constants.planet_info.w;

        vec3 person_pos = camera_ubo.camera_pos.xyz - constants.planet_pos.xyz;
        vec3 sun_pos = constants.sun_pos.xyz - constants.planet_pos.xyz;
        vec3 to_sun = constants.sun_pos.xyz - camera_ubo.camera_pos.xyz;
        float day_frac = (dot(person_pos, sun_pos) / length(person_pos) / length(sun_pos)) * 0.999;
        float solar_angle = (dot(pointCoord, to_sun) / length(to_sun) / length(pointCoord)) * 0.999;
        if (day_frac < 0.0) {
            outColor = texture(skyboxSampler, texCoord);
            return;
        }
        vec2 skyCoord = vec2(
            day_frac,
            -solar_angle
        );

        vec4 skyColor = texture(skycolorSampler, skyCoord);
        vec4 boxColor = texture(skyboxSampler, texCoord);
        alpha *= skyColor.w;
        outColor = (1.0 - alpha) * boxColor + alpha * skyColor;
        outColor.w = 1.0;
    } else {
        outColor = texture(skyboxSampler, texCoord);
    }
}