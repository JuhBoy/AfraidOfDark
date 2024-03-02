#version 330 core

layout (location = 0) in vec3 _pos;
layout (location = 1) in vec2 _uvs;

uniform int offsetX;
uniform vec3 color;

out vec2 uvs;

void main()
{
    gl_Position = vec4(_pos, 1.0);
    uvs = _uvs;
}