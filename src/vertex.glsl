#version 330
in vec3 ms_position;
in vec3 ms_normal;
uniform mat4 view, proj;
out vec3 normal;
void main() {
    normal = ms_normal;
    gl_Position = proj * view * vec4(ms_position, 1.0);
}
