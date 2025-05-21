# Rendering

## Completed

### Viewport & ECS
- [X] Update the viewport size in the OpenGL side
- [X] Pass viewport change from ECS to the rendering system

### Input System
- [X] Add Keyboard input system to handle keyboard events in ECS
	- [X] Implement update_state Keyboard base input system (requires updating state from window (GLFW))

### Testing
- [X] Test viewport changes with a simple game system that responds to input keys

### Frame Buffer & GFX
- [x] Create frame buffer target per camera with ECS binding
	- [X] Rework the `alloc_buffer` from gfx device OpenGL to allow a quad with no indices for rendering the screen shader with 2D vertices
	- [X] Refactor FrameBuffer object to clearly distinguish what is used as screen sampling (consider extracting shaders and buffer objects)

### Camera & Transform
- [X] Update Camera changes from ECS to the rendering system via shader API (uniforms)
- [X] Update Transform of SpriteRenderer2D from ECS to the rendering system via shader API (uniforms)
	- [X] Create TRS matrix for entities (and pass transform data from ECS to rendering)
	- [X] Store camera information to GFX device (size, pixels per unit, etc.)
	- [X] Add clearing color to the camera data
	- [X] Create orthographic projection from camera data
	- [X] Add MVP matrix to shaders and apply it from the CPU side

### Shader API Enhancements
- [X] Add orthographic projection support to the shader API

### Texture & Caching
- [X] Implement a basic caching system for textures
	- [X] Add post-frame rendering texture clears
	- [X] Add frame storage state with reference counting for textures
- [X] Add string-to-GPU handle cache storage to prevent duplicate allocations

## In Progress / Planned

- [X] Implement background color clear using camera settings
- [x] Implement camera frustum culling
  - [x] AABB/OOB simple ECS implementation (No BSP / space partitioning of any kind for now)
- [x] Grid visualisation Debug Tool
  - [x] Math: Create Polyline data structure
  - [x] Rendering: Api to create Shader Storage buffer object
  - [x] Shader: Vertex shader with gl_VertexID to create quad from polyline input
  - [x] Bonus|Shader: Add miter joints for Polylines
  - [x] Bonus|Shader: Add round joints for Polylines
- [ ] Add preserve aspect ratio option for Sprite2D (integrate changes in ECS, Renderer, and Shader)
- [ ] Add sorting layer for Sprite2D (integrate changes in ECS and Renderer)
- [ ] **Small Optimizations**
	- [ ] Use Pixel Buffer Object to update textures when sizes match
	- [ ] Create storage hash function to prevent cloning strings when handling textures

## Future Enhancements

### Rendering Pipeline (Deferred, TBD)
- **Shaders**
	- [ ] Support for Luminance, Diffuse, Specular, and Texture Mapping
	- [ ] Develop a small compiler with header inclusion
- **Light Sources**
	- [ ] Implement Point Light (ECS + Renderer)
	- [ ] Implement Spot Light (ECS + Renderer)
	- [ ] Implement Global Light (ECS + Renderer)
- [ ] Integrate shadow mapping
- [ ] Integrate stencil techniques

- [ ] Other features: Animators, Sprite Atlas, Maps & Scenes serialization

# File System and Profiling

## Future Enhancements

### Profiling & Memory Management
- [ ] Integrate Tracy crate profiler ([Tracy GitHub](https://github.com/wolfpld/tracy?tab=readme-ov-file))
- [ ] Add a custom memory allocator to manage CPU-side memory usage
- [ ] Implement memory & CPU profiling events for Rendering and ECS

### Async File System Operations
- [o] Implement threaded asynchronous loading for textures
    - [x] Implement File load from disk
    - [x] Implement Execution Queue (push & poll)
    - [ ] Implements Polling and Queuing from Renderer.rs
        - [ ] Adds Path config (sent to asset crate)
        - [ ] Remove old code used to load files
- [ ] Implement asynchronous queue for RendererStorage
- [ ] Implement asynchronous rendering for the OpenGL layer

Now use: https://app.asana.com/ for tasks
