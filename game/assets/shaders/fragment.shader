#version 330 core

out vec4 FragColor;
in vec2 uvs;

uniform sampler2D texture0;

void main()
{
    FragColor = texture(texture0, uvs) * vec4(uvs, 0.0, 1.0);
}