#version 330
in vec3 ms_position;
in vec3 ms_normal;
uniform mat4 view, proj, model;
out vec4 normal;
void main() {
    normal = model * vec4(ms_normal, 0.0);
    gl_Position = proj * view * model * vec4(ms_position, 1.0);
}
