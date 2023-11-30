use ratatui::layout::Rect;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;

pub mod board_view;


pub fn centered(container: Rect, width: u16, height: u16) -> Rect {
    let width = if width > container.width { container.width } else { width };
    let height = if height > container.height { container.height } else { height };
    let vertically_centered_rect = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((container.height - height) / 2),
            Constraint::Length(height),
            Constraint::Length((container.height - height) / 2),
        ])
        .split(container)[1];

    let centered_rect = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((container.width - width) / 2),
            Constraint::Length(width),
            Constraint::Length((container.width - width) / 2),
        ])
        .split(vertically_centered_rect)[1];

    centered_rect
}
