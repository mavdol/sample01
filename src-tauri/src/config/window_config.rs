use tauri::{LogicalPosition, LogicalSize, Position, Size, WebviewWindow};

pub fn configure_window_size(window: &WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(Some(monitor)) = window.current_monitor() {
        let size = monitor.size();
        let scale_factor = monitor.scale_factor();

        let screen_logical_width = size.width as f64 / scale_factor;
        let screen_logical_height = size.height as f64 / scale_factor;

        let logical_width = screen_logical_width * 0.95;
        let logical_height = screen_logical_height * 0.95;

        let position_x = (screen_logical_width - logical_width) / 2.0;
        let position_y = (screen_logical_height - logical_height) / 2.0;

        window.set_size(Size::Logical(LogicalSize::new(logical_width, logical_height)))?;
        window.set_position(Position::Logical(LogicalPosition::new(position_x, position_y)))?;
    }

    Ok(())
}
