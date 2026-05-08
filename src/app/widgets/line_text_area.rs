use color_eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{style::Stylize, text::Line, widgets::Widget};

#[derive(Debug, Default)]
pub struct LineTextArea {
    selected: bool,
    buf: String,
    pos: usize,
}

impl LineTextArea {
    pub fn new(buf: String, pos: usize) -> Self {
        debug_assert!(pos <= buf.len());

        Self {
            selected: false,
            buf,
            pos,
        }
    }

    /// Get the internal buffer.
    pub fn get_buf(&self) -> &str {
        &self.buf
    }

    /// Return the selection status.
    pub fn selected(&self) -> bool {
        self.selected
    }

    /// Switches between `true` and `false` on every call.
    ///
    /// ```rust
    /// // ... SNIP ...
    /// assert!(line_text_area_state.toggle_selection());   // true
    /// assert!(!line_text_area_state.toggle_selection());  // false
    /// ```
    ///
    /// Returns the new selection state.
    pub fn toggle_selection(&mut self) -> bool {
        self.selected = !self.selected;

        self.selected
    }

    /// Select the Widget.
    pub fn select(&mut self) {
        self.selected = true;
    }

    /// Deselect the Widget.
    pub fn deselect(&mut self) {
        self.selected = false;
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }

    pub fn handle_event(&mut self, event: &Event) -> Result<()> {
        match event {
            Event::Key(key_event) => self.handle_key_event(key_event)?,
            _ => {}
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> Result<()> {
        match key_event.kind {
            KeyEventKind::Press => match key_event.code {
                KeyCode::Char(ch) => {
                    if ch.is_ascii() && self.pos <= self.buf.len() {
                        self.buf.insert(self.pos, ch);
                        self.pos += 1;
                        tracing::debug!("Inserted {} into {} at {}", ch, self.buf, self.pos - 1);
                    }
                }
                KeyCode::Backspace => {
                    if self.pos > 0 {
                        let _ = self.buf.remove(self.pos - 1);
                        self.pos -= 1;
                    };
                }
                KeyCode::Delete => {
                    if self.pos < self.buf.len() {
                        let _ = self.buf.remove(self.pos);
                    };
                }
                KeyCode::Left => {
                    if self.pos > 0 {
                        self.pos -= 1;
                    };
                }
                KeyCode::Right => {
                    if self.pos < self.buf.len() {
                        self.pos += 1;
                    };
                }
                KeyCode::Up | KeyCode::PageUp | KeyCode::Home => {
                    self.pos = 0;
                }
                KeyCode::Down | KeyCode::PageDown | KeyCode::End => {
                    self.pos = self.buf.len();
                }
                KeyCode::Esc => self.deselect(),
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }
}

impl Widget for &LineTextArea {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let line = if !self.buf.is_empty() {
            let before = self
                .buf
                .get(0..self.pos)
                .expect("Line Text Area: Failed to get slice before pointer.");
            let cursor = self.buf.get(self.pos..self.pos + 1).unwrap_or(" ");
            let after = self.buf.get(self.pos + 1..self.buf.len()).unwrap_or("");

            let cursor = if self.selected {
                cursor.black().on_white()
            } else {
                cursor.into()
            };

            Line::from(vec![before.into(), cursor, after.into()])
        } else {
            Line::from(vec![" ".black().on_white()])
        }
        .left_aligned();

        line.render(area, buf);
    }
}
