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
[x] - Update Transform of SpriteRenderer2D from ECS to the rendering system via shader_api (uniforms)
	[x] Create TRS matrix for entities (+ pass transform data from ECS to rendering)
	[x] Store camera information to GFX device (Size, pixel per unit, etc..)
	[x] Adding clearing color to the camera data 
	[x] Create orthographic projection from camera data 
	[x] Adds MVP matrix to shaders
	[x] Apply MPV matrix from CPU side
[x] - Adds orthographic projection to the shader_api
[x] Implements caching system (super naive) for textures
	[x] adds post frame rendering texture clears
	[x] adds frame storage state with reference counting for textures
[x] Add String to Gpu Handle cache storage to prevent duplicates allocation
[ ] Implements background color clear using camera settings
[ ] Implements camera Frustrum culling
[ ] Adds preserve aspect ratio option for Sprite2D (ECS + Renderer + Shader)
[ ] Adds sorting layer for Sprite2D (ECS + Renderer)
[ ] Small Optims => (shader sharing, PriorityQueue sorting)
	[ ] Use Pixel buffer object to update texture when size match
	[ ] create storage hash function to prevent cloning String when dealing with texture
[ ] File System
	[ ] Add threaded async loading for texture
	[ ] Implements async Queue for RendererStorage
	[ ] Implements async Rendering for OpenGL layer
[ ] Rendering Pipeline (TBD: probably deferred)
	[ ] Shaders
		[ ] Luminance, Diffuse, Specular, Texture Mapping
		[ ] Small compiler with header inclusion
	[ ] Light sources
		[ ] Point Light (ECS + Renderer)
		[ ] Spot Light (ECS + Renderer)
		[ ] Global Light (ECS + Renderer)
	[ ] Shadow map
	[ ] Stencil
[ ] ... Animators, Sprite Atlas, Maps & Scenes serialization

[File System and Profiling]
[ ] Implement Tracy crate profiler
	- https://github.com/wolfpld/tracy?tab=readme-ov-file 
[ ] Adds memory allocator in order to control memory size (CPU side)
[ ] Implement memory & CPU profiling events (Rendering, ECS)