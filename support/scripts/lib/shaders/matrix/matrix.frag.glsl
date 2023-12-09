#version 450

struct Params {
    vec2 uResolution;
    float uTime;
};

layout (location = 0) out vec4 outColor;
layout (binding = 0) uniform Block {
    Params params;
};

precision highp float;

layout (binding = 1) uniform sampler2D tex;

void main()
{
    vec2 uv = gl_FragCoord.xy / params.uResolution;
	float time = params.uTime * 0.4;

	// apply pixelate effect
	// vec2 uv_pixel = uv;
	vec2 uv_pixel = floor(uv * (params.uResolution/4)) / (params.uResolution/4);

    vec4 col1 = vec4(0.510, 0.776, 0.486, 1.0);
    vec4 col2 = vec4(0.200, 0.604, 0.318, 1.0);
    vec4 col3 = vec4(0.145, 0.490 ,0.278, 1.0);
    vec4 col4 = vec4(0.059, 0.255, 0.251, 1.0);

	// displacement on top of y
	vec3 displace = texture(tex, vec2(uv_pixel.x, (uv_pixel.y + time) * 0.05)).xyz;
	displace *= 0.5;
	displace.x -= 1.0;
	displace.y -= 1.0;
	displace.y *= 0.5;

	// color
	vec2 uv_tmp = uv_pixel;
	uv_tmp.y *= 0.2;
	uv_tmp.y += time;
    vec4 color = texture(tex, uv_tmp + displace.xy);

    // match to colors
    vec4 noise = floor(color * 10.0) / 5.0;
    vec4 dark   = mix(col1, col2, uv.y);
    vec4 bright = mix(col3, col4, uv.y);
    color = mix(dark, bright, noise);

	// add gradients (top dark and transparent, bottom bright) 
    float inv_uv = 1.0 - uv_pixel.y;
    color.xyz -= 0.45 * pow(uv_pixel.y, 8.0);
    color.a -= 0.2 * pow(uv_pixel.y, 8.0);
    color += pow(inv_uv, 8.0);

    // make waterfall transparent
    color.a -= 0.2;

    outColor = vec4(color);
}
