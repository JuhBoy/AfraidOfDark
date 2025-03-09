// TODO: 
// I'm currently working on transfering changes done in the ECS part to the underlying system of rendering (here)
// so the update callback must be kill and sent to the opengl side (as it use glViewport() function to update the viewport size)

files:
- opengl.rs
- world.rs (flush_camera_changes)

[Rendering]
[X] - Update the viewport size in the opengl side
[X] - Pass viewport change from ECS to the rendering system
[x] - Add Keyboard input system to handle keyboard events in ECS
				[x] Writting update_state Keybaord base input system (require to update state from window glfw) 
[x] - Test view port changes with a simple game system that respond to a input keys
[o] - Create frame buffer target per camera with ECS binding
			[x] Actually neededing to rework the alloc_buffer form gfx device opengl to allow quad with no indices for rendering the screen shader with 2D vertex
			[x] Refacto FrameBuffer object to make it more clear what is used as screen sampling (probably needs to extract the shaders and buffers object)
[x] - Update Camera changes from ECS to the rendering system via shader_api (uniforms)
[o] - Update Transform of SpriteRenderer2D from ECS to the rendering system via shader_api (uniforms)
	[x] Create TRS matrix for entities (+ pass transform data from ECS to rendering)
	[ ] Store camera information to GFX device (Size, pixel per unit, etc..)
	[ ] Create orthographic projection from camera data 
	[ ] Adds MVP matrix to shaders
	[ ] Apply MPV matrix from CPU side
[ ] - Adds orthographic projection to the shader_api

[File System and Profiling]
[ ] Implement Tracy crate profiler
	- https://github.com/wolfpld/tracy?tab=readme-ov-file 
[ ] Adds memory allocator in order to control memory size (CPU side)
[ ] Implement memory & CPU profiling events (Rendering, ECS)