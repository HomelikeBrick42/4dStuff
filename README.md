# 4D Rendering

This is a 4d sandbox that im working on, it currently supports:

- Volume View (stolen from the 4D Golf game)

## Controls

| Key(s)                                                        | Behavour                                                                                                                                            |
| ------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| Escape                                                        | Toggle mouse lock                                                                                                                                   |
| W/S                                                           | Move foward/backwards along the X axis relative to the camera                                                                                       |
| A/D                                                           | Move left/right along the Z axis relative to the camera                                                                                             |
| Q/E                                                           | Move down/up along the Y axis relative to the camera                                                                                                |
| R/F                                                           | Move ana/kata along the W axis relative to the camera                                                                                               |
| Mouse right/left (when mouse is locked)                       | Rotate in the xz plane relative to the camera                                                                                                       |
| Mouse up/down (when mouse is locked)                          | Rotate in the xy plane relative to the camera (when not in volume view this rotation will not effect any other rotations and is applied afterwards) |
| Mouse scroll (when mouse is locked) (when not in volume view) | Rotate in the zw plane relative to the camera                                                                                                       |
| Mouse scroll (when mouse is locked) (when in volume view)     | Rotate in the yz plane relative to the camera                                                                                                       |
| V                                                             | Toggle volume view                                                                                                                                  |

## What is volume view?

Volume view removes all xy rotation from the camera, and then adds a 90 degree rotation in the yw plane.

This effectively turns the Y axis into the W axis so you can see everything "horizontal" to you in 4D by looking around. (Note that the "W axis" in volume view (the one you cant see) is the _negative_ Y axis, because of the rotation)

You can only see objects in volume mode that are on the same Y level as you.

All objects that are aligned with the imaginary horizontal line going through the middle of your screen in volume view will be visible when leaving volume view.
