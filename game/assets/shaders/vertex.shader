#version 330 core

layout (location = 0) in vec3 pos;

uniform int offsetX;
uniform vec3 color;
uniform sampler2D texture1;

void main()
{
    gl_Position = vec4(pos.x + offsetX, pos.y, pos.z, 1.0);
}