#version 330 core

out vec4 FragColor;

in vec2 uvs;
in vec4 color;

uniform sampler2D texture0;

void main()
{
     vec4 texColor = texture(texture0, uvs) * color;
		 FragColor = texColor;
}