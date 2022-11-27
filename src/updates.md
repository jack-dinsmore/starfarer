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

## June 19

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

## July 8

Transitioned to new machine and made program run on mac. Still needs to figure out mouse camera steering.

## July 9

Solved camera steering problem: on macos: moving the cursor adds to the device mouse delta so that the action is canceled.

## July 11

Skybox

## July 12

Ship model loaded in and started working on ship backend code.

## July 14

Accessory loading. Also rehashed ship model loading so that it's done through a loader which can be more easily generalized to mid-game loading and loading of local files. Also redid the ship model and accessories.

## July 17

Terrain generation & used faster hash functions

## July 28

Collisions, GJK algorithm, and terrain collisions. I had not updated this file for a few days.
Export with Y forward, Z up.
Terrain collision
Collisions put in model file

## Sept 15

Fixed the GJK algorithm so that it computes the collision result without assuming that the answer is face-vertex or edge-edge

## Oct 7

Added the sky and improved planetary collision

## Oct 9

Revamped lepton so that descriptor sets are now tiered by the frequency of their update (shader level, "environment" level, and model level). Environment is unimplemented, but eventually I will do it for shared assets. Either each model will own an `Arc` of the asset and load it if the asset is unloaded, or manually loading, unloading, and deleting the asset will be necessary.

## Oct 16

Sky texture and fixed the bug with multiple sets not being accessible to the shader.

## Oct 29

Added the sun

## Oct 30

Made the sun bright and fixed up the sky.

## Nov 25

Made a lot of progress on the normal map code. Still need to pass in the strength of the adjustment and implement the normal altering, and I also need to fix the jagged textures issue.

# To do

## Features
- Docking with the other ship
- Orbital display

## Fixes
- Implement Environment input level so that double-loaded ships share textures
- Models can be replaced instead of just destroyed and remade
- For speed, reduce memory accesses in the draw command (to the hash table for object and for model and for the vector of draw states)