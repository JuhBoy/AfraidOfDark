#version 430 core

layout (location = 0) in vec3 pos;
layout (location = 1) in vec2 texCoords;

out vec2 coords;

void main() {
	gl_Position = vec4(pos.xy, 0.0, 1.0);
	coords = texCoords;
}