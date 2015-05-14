#version 330
in vec3 ms_position;
uniform mat4 view, proj;
void main() {
   gl_Position = proj * view * vec4(ms_position, 1.0);
}
