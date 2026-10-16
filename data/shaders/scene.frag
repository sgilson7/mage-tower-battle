#version 300 es
precision highp float;
// Three flat bands of light, like the journal's mockups; ink is a flat
// colour; glow is added. Every number a look depends on is a uniform the
// core sends (Builder D3).
in vec3 vNormal;
in vec4 vColor;
uniform float uPass;
uniform vec3 uLight;
uniform vec3 uBands;
uniform vec2 uCuts;
uniform vec3 uSky;
uniform vec2 uGlow;
out vec4 outColor;
void main() {
  if (uPass == 1.0) {
    outColor = vec4(vColor.rgb, 1.0);
    return;
  }
  vec3 n = normalize(vNormal);
  if (uPass > 1.0) {
    // glow: brightest where the halo faces the eye's direction of light
    float rim = abs(dot(n, normalize(uLight)));
    outColor = vec4(vColor.rgb * vColor.a * pow(rim, uGlow.y) / uGlow.x, 1.0);
    return;
  }
  float d = dot(n, normalize(uLight));
  float level = d < uCuts.x ? uBands.x : (d < uCuts.y ? uBands.y : uBands.z);
  vec3 lit = vColor.rgb * level + uSky * (1.0 - level);
  outColor = vec4(mix(lit, vColor.rgb, vColor.a), 1.0);
}
