#version 330
in vec3 normal;
out vec4 color;
void main() {
   vec3 light = vec3(1.0, 1.0, -1.0);
   color = vec4(0.5, 0.5, 0.5, 1.0)*dot(light, normal);
}

