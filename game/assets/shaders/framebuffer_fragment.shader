#version 430 core

out vec4 FragColor;

in vec2 coords;

uniform sampler2D screenTex;

void main() {
	FragColor = vec4(texture(screenTex, coords).rgb, 1.0);
}