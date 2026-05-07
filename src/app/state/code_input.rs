use crossterm::event::{self, KeyCode};
use friend_code::FriendCode;
use ratatui::{
    layout::Alignment,
    style::Stylize,
    text::Line,
    widgets::{Block, BorderType, Widget},
};

use crate::app::state::{AppState, State};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeInput;

impl State for CodeInput {
    fn handle_event(
        &self,
        ctx: &mut crate::app::context::AppContext,
        event: &crossterm::event::Event,
    ) -> color_eyre::Result<()> {
        if ctx.addr_input.selected() {
            if let event::Event::Key(key_event) = &event
                && key_event.code == KeyCode::Enter
            {
                let buf = ctx.addr_input.get_buf().to_string();
                if let Ok(fc) = FriendCode::from_string_friend_code(&buf) {
                    ctx.vchat.add_address(fc.into_socket_addr());
                    ctx.addr_input.clear();
                }

                ctx.to_state(AppState::app());
            } else {
                ctx.addr_input.handle_event(&event)?;
            };
        } else {
            ctx.to_state(AppState::app());
        };

        Ok(())
    }

    fn render(
        &self,
        ctx: &crate::app::context::AppContext,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
    ) {
        let title = Line::from(" Input Friend Code ").bold().red();
        let mut text_area_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(title)
            .title_alignment(Alignment::Left);

        if matches!(ctx.get_state(), AppState::CodeInput(_)) {
            let instructions = Line::from(vec![" Exit Input".into(), " <ESC> ".bold().yellow()]);
            text_area_block = text_area_block.title_bottom(instructions);
        };

        let text_area_block_area = text_area_block.inner(area);
        text_area_block.render(area, buf);

        ctx.addr_input.render(text_area_block_area, buf);
    }
}
