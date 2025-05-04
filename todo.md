[ ] Test with a real project!

[ ] Some way to split the rendering
    . Option 1: HBlank "interrupts", a closure that can run arbitrary code on the videochip at the end of each scaline. Very powerful and open ended, but doesn't allow vertical splits.
    . Option 2: A secondary view rect, renders a portion of the BG Map with alternate scroll values.
    . Option 3: Second BG Map layer?
    Going to try option 1 but with a custom HBlank position, so that the code can run before the end of the line.

[x] crop_x and crop_y, as a way to allow sprites to disappear under the left and top edges.
    . Adds to scroll_x and scroll_y, but max out at 256 - width and MAX_LINES - height

[x] Scanline cache in iterator

[x] FG Flag for BG tiles

[x] Flip and rotate flags for BG Tiles

[x] 3 bit per channel RGB
