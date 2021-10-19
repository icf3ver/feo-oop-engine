#version 450

layout(location = 0) in vec2 position;

// layout(location = 0) out vec2 v_screen_coords;

void main() {
    // v_screen_coords = position;
    
    gl_Position = vec4(position, 0.0, 1.0);
}