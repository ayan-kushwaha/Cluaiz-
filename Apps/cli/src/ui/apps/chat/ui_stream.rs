use ratatui::{
    layout::{Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Paragraph, Wrap, Widget},
    buffer::Buffer,
};
use crate::core::state::AppState;

pub fn render_stream(app: &AppState, area: Rect, buf: &mut Buffer) {
    let mut history_lines: Vec<Line> = Vec::new();
    
    let left_margin = 0; // Use full width in primary flow
    let view_width = area.width;

    for (sender, msg) in &app._chat_history {
        let (color, prefix) = match sender.as_str() {
            "USER"   => (Color::Cyan, "👤 USER"),
            "ARCHER" => (Color::Yellow, "🧬 ARCHER"),
            "CORE"   => (Color::Magenta, "🧬 CORE"),
            "SYSTEM" => (Color::DarkGray, "⚙️ SYNC"),
            "ERROR"  => (Color::Red, "⚠ FAIL"),
            _        => (Color::White, sender.as_str()),
        };

        history_lines.push(Line::from(vec![
            Span::styled(format!(" {} ", prefix), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(Color::Rgb(60, 60, 70))),
            Span::styled(msg, Style::default().fg(Color::Rgb(200, 200, 210))),
        ]));
        history_lines.push(Line::from("")); // Spacer
    }

    let history_len = history_lines.len() as u16;
    let visible_height = area.height.saturating_sub(2);
    let scroll = if history_len > visible_height { history_len - visible_height } else { 0 };

    Paragraph::new(history_lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0))
        .render(Rect::new(area.x + left_margin, area.y, view_width, visible_height), buf);

    // ── PROMPT AREA ──────────────────────────────────────────────────
    let is_loading = app.is_deleting || !app.neural_engine.is_loaded;
    let prompt_char = if is_loading { "…" } else { "❯" };
    let prompt_color = if is_loading { Color::Yellow } else { Color::Cyan };

    let prompt_line = Line::from(vec![
        Span::styled(format!(" {} ", prompt_char), Style::default().fg(prompt_color).add_modifier(Modifier::BOLD)),
        Span::styled(if is_loading { "Initializing Neural Weights..." } else { &app._chat_input }, Style::default().fg(if is_loading { Color::DarkGray } else { Color::White })),
    ]);

    Paragraph::new(prompt_line).render(Rect::new(area.x + left_margin, area.bottom().saturating_sub(1), view_width, 1), buf);
}
