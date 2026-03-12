use crate::plugins::render::graph::CompiledPassDescriptor;
use anyhow::Result;

pub fn ensure_compiled_pass_is_supported(pass: &CompiledPassDescriptor) -> Result<()> {
    match pass {
        CompiledPassDescriptor::Compute(_) => Ok(()),
        CompiledPassDescriptor::Fullscreen(_) => Ok(()),
        CompiledPassDescriptor::Copy(_) => Ok(()),
        CompiledPassDescriptor::Present(_) => Ok(()),
        CompiledPassDescriptor::BuiltinUiComposite(_) => Ok(()),
        CompiledPassDescriptor::Graphics(_) => Ok(()),
    }
}
