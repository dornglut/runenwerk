use crate::plugins::render::graph::CompiledPassExecutionPlan;
use anyhow::Result;

pub fn ensure_compiled_pass_is_supported(pass: &CompiledPassExecutionPlan) -> Result<()> {
    match pass {
        CompiledPassExecutionPlan::Compute(_) => Ok(()),
        CompiledPassExecutionPlan::Fullscreen(_) => Ok(()),
        CompiledPassExecutionPlan::Copy(_) => Ok(()),
        CompiledPassExecutionPlan::Present(_) => Ok(()),
        CompiledPassExecutionPlan::BuiltinUiComposite(_) => Ok(()),
        CompiledPassExecutionPlan::Graphics(_) => Ok(()),
    }
}
