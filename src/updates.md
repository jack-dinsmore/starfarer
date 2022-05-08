## April 23

Jack added text rendering to main.rs, but rendering text covers the triangle we're drawing. I suspect this is the fault of a "uniform_matrix_4_f32_slice" call in the rendering function, which may be messing with the culling procedure.

One we get the triangle to be 3D, we should come back to this. Try enabling / disabling culling & the depth buffer, or clearing the depth buffer.