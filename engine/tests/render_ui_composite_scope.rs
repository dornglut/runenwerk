use engine::plugins::render::{
    CompiledBuiltinImport, CompiledPassExecutionPlan, CompiledResourceRef,
    CompiledUiCompositeExecutionPlan, CompiledViewMask, RenderFlow, compile_flow_plan,
};

fn compiled_ui_pass(main_surface_only: bool) -> CompiledUiCompositeExecutionPlan {
    let flow = RenderFlow::new("ui.scope")
        .with_builtin_ui()
        .builtin_ui_composite_pass("ui")
        .expect("builtin UI pass authoring should succeed");
    let flow = if main_surface_only {
        flow.main_surface_only().finish()
    } else {
        flow.finish()
    };
    let flow = flow.validate().expect("builtin UI flow should validate");
    let compiled = compile_flow_plan(&flow).expect("builtin UI flow should compile");

    compiled
        .execution
        .passes
        .into_iter()
        .find_map(|pass| match pass {
            CompiledPassExecutionPlan::BuiltinUiComposite(pass) => Some(pass),
            _ => None,
        })
        .expect("compiled flow should contain the builtin UI pass")
}

#[test]
fn builtin_ui_composite_preserves_public_all_views_default() {
    let pass = compiled_ui_pass(false);

    assert_eq!(pass.view_mask, CompiledViewMask::AllViews);
    assert_eq!(
        pass.color_output,
        CompiledResourceRef::ImportedBuiltin(CompiledBuiltinImport::SurfaceColor)
    );
}

#[test]
fn builtin_ui_composite_keeps_main_surface_scope_explicit() {
    let pass = compiled_ui_pass(true);

    assert_eq!(pass.view_mask, CompiledViewMask::MainSurfaceOnly);
    assert_eq!(
        pass.color_output,
        CompiledResourceRef::ImportedBuiltin(CompiledBuiltinImport::SurfaceColor)
    );
}
