use crate::plugins::render::graph::CompiledPassDescriptor;
use anyhow::{Result, bail};

pub fn ensure_compiled_pass_is_supported(pass: &CompiledPassDescriptor) -> Result<()> {
    match pass {
        CompiledPassDescriptor::Compute(_) => Ok(()),
        CompiledPassDescriptor::Fullscreen(_) => Ok(()),
        CompiledPassDescriptor::Copy(value) => bail!(
            "copy pass '{}' is declared but backend copy execution is not implemented",
            value.node.id.as_str()
        ),
        CompiledPassDescriptor::Present(value) => bail!(
            "present pass '{}' is declared but backend present execution is not implemented",
            value.node.id.as_str()
        ),
        CompiledPassDescriptor::BuiltinUiComposite(_) => Ok(()),
        CompiledPassDescriptor::Graphics(value) => bail!(
            "graphics pass '{}' is declared but backend graphics execution is not implemented",
            value.node.id.as_str()
        ),
    }
}
