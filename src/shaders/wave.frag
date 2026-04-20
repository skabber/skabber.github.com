#version 300 es
precision highp float;

uniform float u_time;
uniform vec2  u_resolution;
uniform vec2  u_cellSize;
uniform vec2  u_mouse;
uniform float u_mouseStrength;
uniform vec4  u_pulses[10];   // (x, y, birthTime, sign)
uniform int   u_pulseCount;
uniform vec3  u_trail[16];    // (x, y, birthTime)
uniform int   u_trailCount;
uniform sampler2D u_atlas;
uniform int   u_atlasCols;
uniform int   u_charCount;

out vec4 fragColor;

float base_wave(vec2 cell, float t) {
    float px = cell.x / 18.0;
    float py = cell.y / 9.0;
    float w1 = sin(px              + t * 0.80);
    float w2 = sin(py * 1.3        + t * 0.55);
    float w3 = sin(px * 0.9 + py * 0.7 + t * 1.05);
    float w4 = cos(px * 0.7 - py * 1.1 + t * 0.70);
    float cx = 2.5 + sin(t * 0.22) * 1.2;
    float cy = 2.0 + cos(t * 0.17) * 0.9;
    float d1 = distance(vec2(px, py), vec2(cx, cy));
    float w5 = sin(d1 * 1.8 - t * 1.40);
    float d2 = distance(vec2(px, py), vec2(1.0, 4.0));
    float w6 = cos(d2 * 2.1 - t * 0.95);
    float raw = (w1 + w2 + w3 + w4 * 0.8 + w5 * 0.65 + w6 * 0.5) / 4.75;
    return raw * 0.5 + 0.5;
}

float mouse_wave(vec2 cell, float t, vec2 m) {
    float d     = distance(cell, m);
    float ring  = sin(d * 0.55 - t * 2.5);
    float decay = exp(-d * 0.13);
    return ring * decay * 0.55;
}

float click_wave(vec2 cell, float t, int count) {
    float total = 0.0;
    for (int i = 0; i < 10; i++) {
        if (i >= count) break;
        vec4 p = u_pulses[i];
        float age = t - p.z;
        if (age < 0.0 || age > 4.0) continue;
        float d      = distance(cell, p.xy);
        float radius = age * 11.0;
        float diff   = d - radius;
        float ring   = exp(-(diff * diff / 4.0));
        float fade   = exp(-age * 0.7);
        total += ring * fade * p.w;
    }
    return clamp(total, -1.0, 1.0);
}

float trail_wave(vec2 cell, float t, int count) {
    float total = 0.0;
    for (int i = 0; i < 16; i++) {
        if (i >= count) break;
        vec3 p = u_trail[i];
        float age = t - p.z;
        if (age < 0.0 || age > 0.8) continue;
        float d     = distance(cell, p.xy);
        float decay = exp(-d * 0.28);
        float fade  = 1.0 - age / 0.8;
        total += decay * fade;
    }
    return total * 0.18;
}

// Replaces the Rust lcg_rand — produces uniform-ish noise per (cell, frame).
float hash13(vec3 p) {
    p  = fract(p * 0.1031);
    p += dot(p, p.yzx + 33.33);
    return fract((p.x + p.y) * p.z);
}

vec3 hsl_to_rgb(float h, float s, float l) {
    h = mod(h, 360.0) / 60.0;
    float c = (1.0 - abs(2.0 * l - 1.0)) * s;
    float x = c * (1.0 - abs(mod(h, 2.0) - 1.0));
    vec3 rgb;
    if      (h < 1.0) rgb = vec3(c, x, 0.0);
    else if (h < 2.0) rgb = vec3(x, c, 0.0);
    else if (h < 3.0) rgb = vec3(0.0, c, x);
    else if (h < 4.0) rgb = vec3(0.0, x, c);
    else if (h < 5.0) rgb = vec3(x, 0.0, c);
    else              rgb = vec3(c, 0.0, x);
    return rgb + vec3(l - c * 0.5);
}

void main() {
    // gl_FragCoord has origin at bottom-left; our grid is top-left-origin so
    // the wave math matches the original CPU port exactly.
    vec2 frag  = vec2(gl_FragCoord.x, u_resolution.y - gl_FragCoord.y);
    vec2 cellF = frag / u_cellSize;
    vec2 cell  = floor(cellF);
    vec2 local = fract(cellF);

    float bw   = base_wave(cell, u_time);
    float mw   = mouse_wave(cell, u_time, u_mouse) * u_mouseStrength;
    float tw   = trail_wave(cell, u_time, u_trailCount);
    float cw   = click_wave(cell, u_time, u_pulseCount);
    float wave = clamp(bw + mw + tw + cw * 0.5, 0.0, 1.0);

    float frame   = floor(u_time * 60.0);
    float noise   = hash13(vec3(cell, frame));
    float blended = clamp(wave * 0.68 + noise * 0.32, 0.0, 1.0);
    int   idx     = clamp(int(blended * float(u_charCount - 1)), 0, u_charCount - 1);

    int   ax         = idx % u_atlasCols;
    int   ay         = idx / u_atlasCols;
    int   atlasRows  = (u_charCount + u_atlasCols - 1) / u_atlasCols;
    vec2  atlasUV    = (vec2(float(ax), float(ay)) + local)
                     / vec2(float(u_atlasCols), float(atlasRows));
    float glyphA     = texture(u_atlas, atlasUV).a;

    // No inner mod on the position term — wrapping it to 0..360 before scaling
    // by 0.18 creates a ~65° hue jump along cell.x*2.3 + cell.y*1.7 = 360k,
    // which reads as a diagonal seam.
    float hue = mod(u_time * 22.0 + (cell.x * 2.3 + cell.y * 1.7) * 0.18
                    + wave * 140.0, 360.0);
    float sat = 0.65 + wave * 0.35;
    float lit = 0.12 + wave * 0.58;
    vec3  rgb = hsl_to_rgb(hue, sat, lit);

    fragColor = vec4(rgb * glyphA, 1.0);
}
