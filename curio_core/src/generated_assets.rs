use curio_core::AssetDatabaseListing;
pub fn generated_assets() -> Vec<(String, i16, AssetDatabaseListing)> {
    vec![

        (
            "assets/compositions/test.comp".to_string(),
            7902,
            AssetDatabaseListing::Embedded(
                include_bytes!("../assets/compositions/test.comp")
                .to_vec()
            ),
        ),

        (
            "assets/compositions/child2.comp".to_string(),
            5722,
            AssetDatabaseListing::Embedded(
                include_bytes!("../assets/compositions/child2.comp")
                .to_vec()
            ),
        ),

        (
            "assets/texture/skybox.png".to_string(),
            5997,
            AssetDatabaseListing::Embedded(
                include_bytes!("../assets/texture/skybox.png")
                .to_vec()
            ),
        ),

        (
            "assets/mesh/Cube4.glb".to_string(),
            2483,
            AssetDatabaseListing::Embedded(
                include_bytes!("../assets/mesh/Cube4.glb")
                .to_vec()
            ),
        ),

        (
            "assets/mesh/test.anim".to_string(),
            3557,
            AssetDatabaseListing::Embedded(
                include_bytes!("../assets/mesh/test.anim")
                .to_vec()
            ),
        ),

        (
            "assets/mesh/Cube3.glb".to_string(),
            7721,
            AssetDatabaseListing::Embedded(
                include_bytes!("../assets/mesh/Cube3.glb")
                .to_vec()
            ),
        ),

        (
            "assets/mesh/Cube2.glb".to_string(),
            1231,
            AssetDatabaseListing::Embedded(
                include_bytes!("../assets/mesh/Cube2.glb")
                .to_vec()
            ),
        ),

        (
            "assets/mesh/Cube.glb".to_string(),
            2665,
            AssetDatabaseListing::Embedded(
                include_bytes!("../assets/mesh/Cube.glb")
                .to_vec()
            ),
        ),

        (
            "assets/mesh/char_grunt.anim".to_string(),
            704,
            AssetDatabaseListing::Embedded(
                include_bytes!("../assets/mesh/char_grunt.anim")
                .to_vec()
            ),
        ),

        (
            "assets/mesh/ground.glb".to_string(),
            4803,
            AssetDatabaseListing::Embedded(
                include_bytes!("../assets/mesh/ground.glb")
                .to_vec()
            ),
        ),

    ]
}
