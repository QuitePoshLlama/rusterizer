Rust software rasterizer inspired by Sebastian Lague https://www.youtube.com/watch?v=yyJ-hdISgnw

Currently can load .obj files with textures and apply simple shading using normal maps 

![a399e4](https://github.com/user-attachments/assets/8fbbb5c2-5925-4d5b-9d81-f96dfedc2175)

TODO:
* [x] Modularize code 
* [x] Multithreading
* [ ] Find way to multithread unsafe writing to shared buffer
* [ ] Min-max performance (SIMD)?
* [ ] Geometry culling?
* [ ] Adapting to write a GPU shader
* [ ] Raytracing
* [ ] Loading/transformming multiple models and arranging scenes
* [ ] User interactable camera
