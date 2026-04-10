use editor_shell::ConsoleViewModel;

pub fn build_console_view_model(lines: &[String]) -> ConsoleViewModel {
    ConsoleViewModel {
        lines: lines.to_vec(),
    }
}
