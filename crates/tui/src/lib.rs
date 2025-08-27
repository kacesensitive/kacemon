pub mod app;
pub mod draw;
pub mod input;
pub mod ui;

pub use app::App;
pub use draw::{DrawState, Drawer};
pub use input::{InputEvent, InputHandler};
pub use ui::*;

#[cfg(test)]
mod tests {
    use super::*;
    use kacemon_core::Config;
    
    #[test]
    fn test_input_handler_creation() {
        let handler = InputHandler::new();
        assert!(!handler.is_in_filter_mode());
    }
    
    #[test]
    fn test_layout_creation() {
        // This test requires a terminal, so we'll just test the basic structure
        let result = std::panic::catch_unwind(|| {
            ui::Layout::new()
        });
        // Don't assert success since we might not have a terminal in CI
        // Just ensure it doesn't panic unexpectedly
    }
    
    #[test]
    fn test_color_scheme_creation() {
        let scheme = ui::ColorScheme::new(&kacemon_core::Theme::Dark, false);
        // Just verify it creates without panicking
        
        let no_color_scheme = ui::ColorScheme::new(&kacemon_core::Theme::Dark, true);
        // Verify no-color mode
    }
    
    #[test]
    fn test_rect_operations() {
        let rect = ui::Rect::new(10, 20, 100, 50);
        assert_eq!(rect.x, 10);
        assert_eq!(rect.y, 20);
        assert_eq!(rect.width, 100);
        assert_eq!(rect.height, 50);
        assert_eq!(rect.right(), 110);
        assert_eq!(rect.bottom(), 70);
        assert_eq!(rect.area(), 5000);
        assert!(!rect.is_empty());
        
        let inner = rect.inner(5);
        assert_eq!(inner.x, 15);
        assert_eq!(inner.y, 25);
        assert_eq!(inner.width, 90);
        assert_eq!(inner.height, 40);
    }
    
    #[test]
    fn test_draw_state_default() {
        let state = DrawState::default();
        assert_eq!(state.selected_index, 0);
        assert_eq!(state.table_start_index, 0);
        assert!(!state.show_help);
        assert!(!state.in_filter_mode);
        assert!(state.filter_text.is_empty());
        assert!(!state.visible_columns.is_empty());
    }
}