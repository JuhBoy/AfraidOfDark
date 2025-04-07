#version 430 core

layout (location = 0) in vec3 _pos;
layout (location = 1) in vec2 _uvs;

uniform vec4 surface_color;
uniform mat4 TRS;
uniform mat4 VIEW;
uniform mat4 PROJ;

out vec2 uvs;
out vec4 color;

void main()
{
    gl_Position = PROJ * VIEW * TRS * vec4(_pos.xyz, 1.0);
    uvs = _uvs;
    color = surface_color;
}