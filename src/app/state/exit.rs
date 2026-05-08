use crossterm::event;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Clear},
};

use crate::app::{
    KEY_CODE_ACCEPT, KEY_CODE_DECLINE,
    state::{AppState, State},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exit;

impl State for Exit {
    fn handle_event(
        &self,
        ctx: &mut crate::app::context::AppContext,
        event: &crossterm::event::Event,
    ) -> color_eyre::Result<()> {
        match event {
            event::Event::Key(key_event) => {
                if key_event.code == KEY_CODE_ACCEPT {
                    ctx.set_exit(true);
                    tracing::info!("Exiting...");
                } else if key_event.code == KEY_CODE_DECLINE {
                    ctx.to_state(AppState::app());
                    tracing::info!("Canceled exiting.");
                };
            }
            _ => {}
        }

        Ok(())
    }

    fn render(
        &self,
        _: &crate::app::context::AppContext,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
    ) {
        let vertical_layout = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(5),
            Constraint::Fill(1),
        ]);
        let [_, vertical_exit_area, _] = vertical_layout.areas(area);

        let middle_layout = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(32),
            Constraint::Fill(1),
        ]);
        let [_, exit_area, _] = middle_layout.areas(vertical_exit_area);

        let title = Line::from(" Exit? ").left_aligned().bold().red();
        let instructions = Line::from(vec![
            " Back".into(),
            " <ESC> ".bold().yellow(),
            " Confirm".into(),
            " <ENTER> ".bold().yellow(),
        ]);
        let block = Block::bordered()
            .border_type(BorderType::Thick)
            .title(title)
            .title_bottom(instructions);
        let block_area = block.inner(exit_area);
        Clear.render(exit_area, buf); // Clear exit_area area so no chars shine throug.
        block.render(exit_area, buf);

        let text = Line::from("Do you want to exit?").centered().red();
        text.render(block_area, buf);
    }
}
