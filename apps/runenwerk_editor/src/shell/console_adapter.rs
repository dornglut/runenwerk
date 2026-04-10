use editor_shell::ConsoleViewModel;

const MAX_CONSOLE_LINES: usize = 12;

pub fn build_console_view_model(lines: &[String]) -> ConsoleViewModel {
    let start = lines.len().saturating_sub(MAX_CONSOLE_LINES);
    ConsoleViewModel {
        lines: lines[start..].to_vec(),
    }
}
