// Owner: Cavern Hunt Material Domain - Tests
#[cfg(test)]
mod tests {
    use super::{
        MaterialGraphAssetV1, MaterialGraphOutputsV1, MaterialNodeKindV1, MaterialNodeV1,
        compile_material_graph,
    };

    #[test]
    fn material_graph_compile_is_deterministic() {
        let asset = MaterialGraphAssetV1 {
            version: 1,
            id: "deterministic".to_string(),
            nodes: vec![
                MaterialNodeV1 {
                    id: "world_pos".to_string(),
                    kind: MaterialNodeKindV1::WorldPos,
                },
                MaterialNodeV1 {
                    id: "world_normal".to_string(),
                    kind: MaterialNodeKindV1::WorldNormal,
                },
                MaterialNodeV1 {
                    id: "noise".to_string(),
                    kind: MaterialNodeKindV1::TriplanarNoise {
                        position: "world_pos".to_string(),
                        normal: "world_normal".to_string(),
                        scale: 0.2,
                        sharpness: 3.0,
                        seed: 11.0,
                    },
                },
                MaterialNodeV1 {
                    id: "tint".to_string(),
                    kind: MaterialNodeKindV1::ConstantVec3 {
                        value: [0.1, 0.2, 0.3],
                    },
                },
                MaterialNodeV1 {
                    id: "noise_vec".to_string(),
                    kind: MaterialNodeKindV1::ToVec3 {
                        input: "noise".to_string(),
                    },
                },
                MaterialNodeV1 {
                    id: "base".to_string(),
                    kind: MaterialNodeKindV1::Add {
                        a: "tint".to_string(),
                        b: "noise_vec".to_string(),
                    },
                },
                MaterialNodeV1 {
                    id: "roughness".to_string(),
                    kind: MaterialNodeKindV1::ConstantScalar { value: 0.7 },
                },
                MaterialNodeV1 {
                    id: "metallic".to_string(),
                    kind: MaterialNodeKindV1::ConstantScalar { value: 0.03 },
                },
            ],
            outputs: MaterialGraphOutputsV1 {
                base_color: "base".to_string(),
                roughness: "roughness".to_string(),
                metallic: "metallic".to_string(),
                normal_perturb: None,
                ao: None,
                emissive: None,
            },
        };

        let first = compile_material_graph(&asset).expect("graph compiles");
        let second = compile_material_graph(&asset).expect("graph compiles");
        assert_eq!(first, second);
    }

    #[test]
    fn invalid_reference_fails_compile() {
        let asset = MaterialGraphAssetV1 {
            version: 1,
            id: "invalid".to_string(),
            nodes: vec![MaterialNodeV1 {
                id: "base".to_string(),
                kind: MaterialNodeKindV1::Add {
                    a: "missing".to_string(),
                    b: "missing_too".to_string(),
                },
            }],
            outputs: MaterialGraphOutputsV1 {
                base_color: "base".to_string(),
                roughness: "base".to_string(),
                metallic: "base".to_string(),
                normal_perturb: None,
                ao: None,
                emissive: None,
            },
        };

        let error = compile_material_graph(&asset).expect_err("graph should fail");
        assert!(error.message.contains("references unknown node"));
    }
}
