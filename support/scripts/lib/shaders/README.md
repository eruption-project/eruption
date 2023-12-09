# Shaders

This directory holds shader programs that are used to render hardware accelerated effects

## Compile

```sh
glslangValidator --target-env vulkan1.2 -l support/scripts/lib/shaders/matrix/*.glsl -o support/scripts/lib/shaders/matrix/matrix.spirv
```
