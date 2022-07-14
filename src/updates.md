## May 9

Started. I'm using the vulkan tutorial [here](https://kylemayes.github.io/vulkanalia/setup/base_code.html), which is written for the [vulkanalia](https://docs.rs/vulkanalia/0.14.0/vulkanalia/) crate, but I'm actually going to use the [ash](https://docs.rs/ash/0.37.0+1.3.209/ash/) crate because it's much more common.

## May 30

- Created lepton and made the Graphics module.
- Added shaders
- Added a camera

## May 31

- Fullscreen & recreate swapchain. Also worked on some ship designs (Starling)
- Designed a method to compute how good RCS placement is for custom-built ships.

## June 5

- Finished docking port texture
- Camera steering with mouse pointer

## June 7

- Pattern reloading now that models have been stored as Rcs.
- Normals & Phong shading

## June 8

- Shader revamping

## June 9

- Made Object class
- Made lepton prelude
- Revamped shaders

## June 10

- Made progress on UI

## June 12

- Push constants
- Multiple shaders for one pattern
- Rerecording of command buffers
- Inputs are no longer static
- Removed reference passes of CommandBuffer and PipelineLayout

## June 14:

I made the UI struct and implemented a single-buffered texture. Performance is horrible in debug mode for a complete UI redraw (5 FPS) but much better in release mode (>60 FPS). Used https://stackoverflow.com/questions/40574668/how-to-update-texture-for-every-frame-in-vulkan as a guide.

Also moved the texture binding to zero instead of 3.

# June 19

Rendering text and drew FPS to screen.

Automatic font file creation

## June 21

Text kerning and single-channel font textures

## June 26

Finished HUD and button pressing

## July 6

Made textures, obj files, and fonts into their own .sfr format and included them in the binary.

## July 7

Reorganized things into a more useful fashion and made a physics engine thread. Built rigid bodies & started astroworks thrace-class station

# July 8

Transitioned to new machine and made program run on mac. Still needs to figure out mouse camera steering.

# July 9

Solved camera steering problem: on macos: moving the cursor adds to the device mouse delta so that the action is canceled.

# July 11

Skybox

# July 12

Ship model loaded in and started working on ship backend code.

## To do:
- Space ships controls
- Make sure that double-loaded ships share textures