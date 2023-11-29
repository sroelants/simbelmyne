use ratatui::layout::Rect;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;

pub mod board_view;


pub fn centered(container: Rect, width: u16, height: u16) -> Rect {
    let vertically_centered_rect = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min((container.height - height) / 2),
            Constraint::Min(height),
            Constraint::Min((container.height - height) / 2),
        ])
        .split(container)[1];

    let centered_rect = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min((container.width - width) / 2),
            Constraint::Min(width),
            Constraint::Min((container.width - width) / 2),
        ])
        .split(vertically_centered_rect)[1];

    centered_rect
}
