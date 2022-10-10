
#version 450

#extension GL_ARB_separate_shader_objects : enable

layout (push_constant) uniform SkyboxPushConstants {
    vec4 planet_pos;
    vec4 sun_pos;
    vec4 planet_info; // Zero if allowed, radius, scale height, min_alpha
} constants;
layout (binding = 0) uniform CameraData {
    mat4 view;
    mat4 proj;
    vec4 camera_pos;
} camera_ubo;

layout (binding = 1) uniform sampler2D skySampler;
layout (binding = 2) uniform sampler2D texSampler;

layout (location = 0) in vec2 texCoord;

layout (location = 0) out vec4 outColor;

void main() {
    if (constants.planet_info.x != 0.0) {
        float alpha = exp(-
        (length(constants.planet_pos - camera_ubo.camera_pos) - constants.planet_info.y)
         / constants.planet_info.z) * constants.planet_info.w;

        outColor = (1.0 - alpha) * texture(texSampler, texCoord) + alpha * vec4(0.5, 0.5, 1.0, 1.0);
    } else {
        outColor = texture(texSampler, texCoord);
    }
}