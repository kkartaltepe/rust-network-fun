#version 330
in vec4 normal;
out vec4 color;
void main() {
   vec3 light = normalize(vec3(0.0, 1.0, -1.0));
   color = vec4(0.5, 0.5, 0.5, 1.0)*dot(light, normal.xyz);
}

