#version 430 core

in vec4 color;
in vec3 world_position;
in vec2 uv;

out vec4 FragColor;

void main()
{
    FragColor = color;
}