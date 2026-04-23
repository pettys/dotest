use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

pub(crate) fn styled_output_lines(output_lines: &[String]) -> Vec<Line<'static>> {
    output_lines
        .iter()
        .map(|l| {
            let style = if l.contains("Passed") || l.starts_with('✓') {
                Style::default().fg(Color::Green)
            } else if l.contains("Failed") || l.starts_with('✗') {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else if l.contains("warning") {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(Span::styled(l.clone(), style))
        })
        .collect()
}

/// Max vertical scroll for output [`Paragraph`] with word wrap: wrapped row count minus viewport rows.
/// Must match how the UI builds the output widget (wrap + styling does not change widths vs plain text).
pub(crate) fn output_wrapped_scroll_max(
    output_lines: &[String],
    inner_width: u16,
    inner_height: u16,
) -> u16 {
    if inner_width == 0 || inner_height == 0 {
        return 0;
    }
    let paragraph = Paragraph::new(styled_output_lines(output_lines)).wrap(Wrap { trim: false });
    let total_rows = paragraph.line_count(inner_width.max(1));
    total_rows
        .saturating_sub(inner_height as usize)
        .min(u16::MAX as usize) as u16
}

pub(super) fn format_elapsed(d: std::time::Duration) -> String {
    let s = d.as_secs();
    format!("{}:{:02}", s / 60, s % 60)
}

pub(super) fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
