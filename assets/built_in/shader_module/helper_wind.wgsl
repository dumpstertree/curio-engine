//==============================================================================
//
// Wind Helpers
//
//==============================================================================

//------------------------------------------------------------------------------
// Travelling Gust Field
//
// Produces large rolling waves that move through the world.
//
//------------------------------------------------------------------------------
fn gust_field(
    world_pos : vec3<f32>,
    phase_offset : f32
) -> f32 {

    let phase =
        wind_phase(world_pos)
        + domain_warp(world_pos)
        + phase_offset;

    let gust =
        sin(phase)
        +
        0.45 * sin(phase * 2.1 + 0.7)
        +
        0.20 * sin(phase * 4.3);

    return gust / 1.65;
}


//------------------------------------------------------------------------------
// Small Leaf Flutter
//
// Faster, higher-frequency motion.
//
//------------------------------------------------------------------------------
fn flutter_field(
    world_pos : vec3<f32>,
    phase_offset : f32
) -> f32 {

    let p = world_pos.xz;

    let t =
        vegetation.parameters.x * FLUTTER_SPEED;

    return
        sin(
            p.x * 1.9 +
            p.y * 2.4 +
            t +
            phase_offset
        );
}


//------------------------------------------------------------------------------
// Bend Calculation
//
// Combines gusts and flutter.
//
//------------------------------------------------------------------------------
fn wind_offset(
    world_pos : vec3<f32>,
    wind_weight : f32,
    phase_offset : f32
) -> vec3<f32> {

    let gust =
        gust_field(world_pos, phase_offset);

    let flutter =
        flutter_field(world_pos, phase_offset);

    let amount =
        vegetation.parameters.y
        * wind_weight;

    let direction =
        normalize(
            vegetation.wind.xy
        );

    let horizontal =
        direction
        *
        (
            gust
            +
            flutter * 0.18
        )
        *
        amount;

    // slight vertical bob
    let vertical =
        abs(gust)
        * 0.04
        * amount;

    return vec3<f32>(
        horizontal.x,
        vertical,
        horizontal.y
    );
}


//==============================================================================
//
// Vertex Shader
//
//==============================================================================

@vertex
fn vs_main(
    model : VertexInput,
    instance : InstanceInput
) -> VSOut {

    //----------------------------------------------------------------------
    // Build model matrix
    //----------------------------------------------------------------------

    let model_matrix = mat4x4<f32>(
        instance.model0,
        instance.model1,
        instance.model2,
        instance.model3
    );

    let normal_matrix = mat3x3<f32>(
        model_matrix[0].xyz,
        model_matrix[1].xyz,
        model_matrix[2].xyz
    );

    //----------------------------------------------------------------------
    // World Position
    //----------------------------------------------------------------------

    var world_position =
        model_matrix
        * vec4<f32>(
            model.position,
            1.0
        );

    //----------------------------------------------------------------------
    // Wind Data
    //----------------------------------------------------------------------

    let wind_weight =
        saturate(model.color.r);

    let phase_offset =
        model.color.g * PI * 2.0;

    //----------------------------------------------------------------------
    // Rooted Bending
    //
    // Root remains fixed.
    // Tip receives full motion.
    //----------------------------------------------------------------------

    let bend =
        wind_offset(
            world_position.xyz,
            wind_weight,
            phase_offset
        );

    world_position.xyz += bend;

    //----------------------------------------------------------------------
    // Normal Adjustment
    //
    // Rotate the lighting slightly toward the bend.
    // Prevents leaves from appearing detached from lighting.
    //----------------------------------------------------------------------

    var world_normal =
        normalize(
            normal_matrix
            * model.normal
        );

    world_normal =
        normalize(
            world_normal
            +
            vec3<f32>(
                bend.x,
                0.0,
                bend.z
            )
            * 0.45
        );

    //----------------------------------------------------------------------
    // Output
    //----------------------------------------------------------------------

    var out : VSOut;

    out.clip_position =
        camera.view_proj
        * world_position;

    out.world_position =
        world_position.xyz;

    out.world_normal =
        world_normal;

    out.shadow_position =
        shadow_camera.light_view_proj
        * world_position;

    out.wind_weight =
        wind_weight;

    out.ao =
        max(
            model.color.b,
            0.15
        );

    // Used later for tip brightening.
    out.local_height =
        model.position.y;

    return out;
}