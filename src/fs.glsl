varying highp vec2 vTextureCoord;
varying highp vec3 vLighting;

uniform sampler2D uSampler;

void main(void) {
    // Get some light on black texel
    highp vec4 texelColor = texture2D(uSampler, vTextureCoord) + vec4(0.01, 0.01, 0.01, 0);
    highp float coeff = 1.0 - texelColor.a;
    texelColor = texelColor + vec4(1.0, 1.0, 1.0, 1.0) * coeff;

    gl_FragColor = vec4(texelColor.rgb * vLighting, texelColor.a);
}