//==============================================================================
//
// Curio Engine
// Ghibli Vegetation Shader
//
// Part 1
// Core Infrastructure
//
//==============================================================================


//==============================================================================
// Camera
//==============================================================================

struct Camera {
    view_pos : vec4<f32>,
    view_proj : mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera : Camera;


//==============================================================================
// Shadow Camera
//==============================================================================

struct ShadowCamera {
    light_view_proj : mat4x4<f32>,
};

@group(3) @binding(0)
var<uniform> shadow_camera : ShadowCamera;


//==============================================================================
// Vegetation Uniforms
//==============================================================================

struct VegetationUniform {

    // xyz = base vegetation color
    base_color : vec4<f32>,

    // x = time
    // y = global wind strength
    // z = variation strength
    // w = translucency strength
    parameters : vec4<f32>,

    // xy = normalized wind direction
    // zw = reserved
    wind : vec4<f32>,
};

@group(0) @binding(0)
var<uniform> vegetation : VegetationUniform;


//==============================================================================
// Lighting
//==============================================================================

struct GpuLight {

    // xyz = position
    // w = light type
    position : vec4<f32>;

    // rgb = color
    // a = intensity
    color_intensity : vec4<f32>;

    // xyz = direction
    // w = radius
    direction_radius : vec4<f32>;
};

struct LightBuffer {

    @align(16)
    count : u32,

    @align(16)
    lights : array<GpuLight,16>;
};

@group(2) @binding(0)
var<storage,read> light_buffer : LightBuffer;


//==============================================================================
// Shadow Map
//==============================================================================

@group(3) @binding(1)
var shadow_map : texture_depth_2d;

@group(3) @binding(2)
var shadow_sampler : sampler_comparison;


//==============================================================================
// Vertex Input
//==============================================================================

struct VertexInput {

    @location(0)
    position : vec3<f32>;

    @location(1)
    normal : vec3<f32>;

    // R = wind weight
    // G = phase offset
    // B = ambient occlusion
    @location(2)
    color : vec3<f32>;

    @location(3)
    uv0 : vec2<f32>;

    @location(4)
    uv1 : vec2<f32>;
};


//==============================================================================
// Instance Input
//==============================================================================

struct InstanceInput {

    @location(5)
    model0 : vec4<f32>;

    @location(6)
    model1 : vec4<f32>;

    @location(7)
    model2 : vec4<f32>;

    @location(8)
    model3 : vec4<f32>;
};


//==============================================================================
// Vertex Output
//==============================================================================

struct VSOut {

    @builtin(position)
    clip_position : vec4<f32>;

    @location(0)
    world_position : vec3<f32>;

    @location(1)
    world_normal : vec3<f32>;

    @location(2)
    shadow_position : vec4<f32>;

    @location(3)
    wind_weight : f32;

    @location(4)
    ao : f32;

    @location(5)
    local_height : f32;
};


//==============================================================================
// Constants
//==============================================================================

const PI = 3.14159265;

const AMBIENT_LIGHT = 0.35;

const SHADOW_STRENGTH = 0.65;

const WIND_SPEED = 0.8;

const FLUTTER_SPEED = 2.5;

const GUST_SCALE = 0.045;

const FLUTTER_SCALE = 0.35;

const EDGE_DARKENING = 0.20;


//==============================================================================
// Utility
//==============================================================================

fn saturate(v : f32) -> f32 {
    return clamp(v,0.0,1.0);
}

fn remap(
    value : f32,
    inMin : f32,
    inMax : f32,
    outMin : f32,
    outMax : f32
) -> f32 {

    return outMin +
        (value-inMin) *
        (outMax-outMin) /
        (inMax-inMin);
}


//==============================================================================
// Cheap Hash
//
// Used everywhere instead of texture noise.
//
// Returns:
//
// 0-1
//
//==============================================================================

fn hash12(p : vec2<f32>) -> f32 {

    let h =
        dot(
            p,
            vec2<f32>(127.1,311.7)
        );

    return fract(sin(h)*43758.5453123);
}


//==============================================================================
// World Variation
//
// Large soft patches.
//
// No visible tiling.
//
//==============================================================================

fn world_variation(
    world_pos : vec3<f32>
) -> f32 {

    let p = world_pos.xz;

    let large =
        sin(p.x*0.06)
      * cos(p.y*0.05);

    let medium =
        sin(p.x*0.18+2.3)
      * sin(p.y*0.21);

    let broad =
        sin((p.x+p.y)*0.025);

    let variation =
        large*0.6 +
        medium*0.3 +
        broad*0.1;

    return variation;
}


//==============================================================================
// Wind Phase
//
// Generates travelling gusts.
//
//==============================================================================

fn wind_phase(
    world_pos : vec3<f32>
) -> f32 {

    let direction =
        normalize(
            vegetation.wind.xy
        );

    let travel =
        dot(
            world_pos.xz,
            direction
        );

    return
        travel*GUST_SCALE
        +
        vegetation.parameters.x*WIND_SPEED;
}


//==============================================================================
// Domain Warp
//
// Breaks perfectly parallel waves.
//
// Extremely cheap.
//
//==============================================================================

fn domain_warp(
    world_pos : vec3<f32>
) -> f32 {

    let p = world_pos.xz;

    return

        sin(p.x*0.11)

        +

        sin(p.y*0.13)

        +

        sin((p.x+p.y)*0.07);
}


//==============================================================================
// Painterly Color Variation
//
// Returns
//
// 0.9 - 1.1
//
//==============================================================================

fn color_variation(
    world_pos : vec3<f32>
) -> f32 {

    let v =
        world_variation(world_pos);

    return

        1.0

        +

        vegetation.parameters.z

        *

        v

        *

        0.08;
}