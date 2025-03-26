# 4D Ray Tracer

This is a 4d sandbox that im working on, it currently supports:

- Hyperspheres
- Hyperplanes
- Translation Gizmos
- Volume View (stolen from the 4D Golf game)

## Controls

| Key(s)                                                        | Behavour                                                                                                                                            |
| ------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| Escape                                                        | Toggle mouse lock                                                                                                                                   |
| Left Click (when mouse is unlocked)                           | Select object or interact with gizmo                                                                                                                |
| V                                                             | Toggle volume view                                                                                                                                  |
| G                                                             | Toggle gizmos being relative to camera rotation                                                                                                     |
| W/S                                                           | Move foward/backwards along the X axis relative to the camera                                                                                       |
| A/D                                                           | Move left/right along the Z axis relative to the camera                                                                                             |
| Q/E                                                           | Move down/up along the Y axis relative to the camera                                                                                                |
| R/F                                                           | Move ana/kata along the W axis relative to the camera                                                                                               |
| Mouse right/left (when mouse is locked)                       | Rotate in the xz plane relative to the camera                                                                                                       |
| Mouse up/down (when mouse is locked)                          | Rotate in the xy plane relative to the camera (when not in volume view this rotation will not effect any other rotations and is applied afterwards) |
| Mouse scroll (when mouse is locked) (when not in volume view) | Rotate in the zw plane relative to the camera                                                                                                       |
| Mouse scroll (when mouse is locked) (when in volume view)     | Rotate in the yz plane relative to the camera                                                                                                       |

## What is volume view?

Volume view removes all xy rotation from the camera, and then adds a 90 degree rotation in the yw plane.

This effectively turns the Y axis into the W axis so you can see everything "horizontal" to you in 4D by looking around. (Note that the "W axis" in volume view (the one you cant see) is the _negative_ Y axis, because of the rotation)
