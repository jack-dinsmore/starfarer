## May 12

I have just pulled from [the tutorial](https://github.com/unknownue/vulkan-tutorial-rust) github, for the [C++ vulkan tutorial](https://vulkan-tutorial.com/) and I have code that compiles but does not show an image. However, I did change some things. It's possible that
    1. The shaders are not compiling properly
    2. Loading them in as u32s and dividing the length of the array by 4 is not appropriate
    3. I mishandled the case matching for image.

I can check the third case by downgrading to the version of image that the original program uses.