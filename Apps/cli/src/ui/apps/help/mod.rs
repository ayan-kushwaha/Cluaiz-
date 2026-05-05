use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
    buffer::Buffer,
};
use crate::core::state::AppState;
use crate::theme::Theme;

pub fn render_widget(_app: &mut AppState, _theme: &Theme, area: Rect, buf: &mut Buffer) {
    let text = "\
📚 OS Core Documentation

[ Navigation ]
- ← / → Arrow   : Switch Terminal Tabs (Chat / Menu)
- ↑ / ↓ Arrow   : Scroll Lists (Roster / Menu)
- Enter         : Deep Select / Open App
- Esc           : Back / Quit Application

[ Philosophies ]
1. Cluaiz Execution   : No telemetry. Your data stays on-device.
2. Cluaiz Intelligence  : local-first models optimized for your hardware.
3. Pure Rust Architecture: Zero-latency, memory-safe, high-performance.

[ Commands ]
/help    - Open this display
/roster  - Manage Model weights
/clear   - Flush chat terminal";

    let widget = Paragraph::new(text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" 📚 Cluaiz Core OS Help "));
    
    widget.render(area, buf);
}
