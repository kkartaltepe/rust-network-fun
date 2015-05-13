#version 330
layout(location = 0) in vec3 position_modelspace;
void main() {
   gl_Position = vec4(position_modelspace, 1.0);
}
