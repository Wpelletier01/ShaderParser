#version 430 core
//
out vec4 FragColor;
//
in vec3 color;
in vec2 TexCoord;
//
//
uniform sampler2D TEX_DIFFUSE_0;
//
//
void main()
{

    FragColor = texture(TEX_DIFFUSE_0,TexCoord);

} 