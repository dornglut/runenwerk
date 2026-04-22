pub mod event;
pub mod focus;
pub mod keyboard;
pub mod pointer;
pub mod routing;
pub mod shortcut;

pub use event::*;
pub use focus::*;
pub use keyboard::*;
pub use pointer::*;
pub use routing::*;
pub use shortcut::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_response_helpers_preserve_default_routing_contracts() {
        let ignored = InputResponse::ignored();
        assert_eq!(ignored.propagation, EventPropagation::Continue);
        assert_eq!(ignored.capture, PointerCapture::None);
        assert_eq!(ignored.focus_change, FocusChange::None);
        assert!(!ignored.repaint);
        assert!(!ignored.relayout);

        let handled = InputResponse::handled();
        assert_eq!(handled.propagation, EventPropagation::Stop);
        assert_eq!(handled.capture, PointerCapture::None);
        assert_eq!(handled.focus_change, FocusChange::None);
        assert!(!handled.repaint);
        assert!(!handled.relayout);
    }
}
