// TODO: 
// I'm currently working on transfering changes done in the ECS part to the underlying system of rendering (here)
// so the update callback must be kill and sent to the opengl side (as it use glViewport() function to update the viewport size)

files:
- opengl.rs
- world.rs (flush_camera_changes)

// [X] - Update the viewport size in the opengl side
// [X] - Pass viewport change from ECS to the rendering system
        [ ] - Test view port changes with a simple game system that respond to a hot key
// [ ] - Update Camera changes from ECS to the rendering system via shader_api (uniforms)
// [ ] - Update Transform of SpriteRenderer2D from ECS to the rendering system via shader_api (uniforms)
// [ ] - Adds orthographic projection to the shader_api