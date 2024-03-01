#version 330 core

layout (location = 0) in vec3 pos;

uniform int offsetX;

void main()
{
    gl_Position = vec4(pos.x + offsetX, pos.y, pos.z, 1.0);
}