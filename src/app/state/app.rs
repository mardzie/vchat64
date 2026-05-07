use color_eyre::Result;
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    style::Stylize,
    text::Line,
    widgets::{Block, BorderType, Borders, List, Widget},
};

use crate::app::{
    context::AppContext,
    state::{AppState, CodeInput, State},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct App;

impl App {
    fn handle_key_event(&self, ctx: &mut AppContext, key_event: &KeyEvent) -> Result<()> {
        match key_event.kind {
            KeyEventKind::Press => match key_event.code {
                KeyCode::Char('q') => {
                    ctx.to_state(AppState::exit());
                    log::debug!("Into `Exit` state.");
                }
                KeyCode::Char('i') => {
                    ctx.to_state(AppState::code_input());
                    ctx.addr_input.select();
                    log::debug!("Into `CodeInput` state.");
                }
                _ => {}
            },
            _ => {}
        };

        Ok(())
    }

    fn render_friend_codes(
        &self,
        ctx: &AppContext,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ]);
        let [
            public_header_area,
            public_code_area,
            _,
            local_header_area,
            local_code_area,
        ] = layout.areas(area);

        let title = Line::from(" Public Friend Code ").bold().yellow();
        let block = Block::new().borders(Borders::TOP).title(title);
        block.render(public_header_area, buf);

        let public_friend_code_line = Line::from(ctx.public_friend_code.to_pretty_string())
            .bold()
            .red()
            .centered();
        public_friend_code_line.render(public_code_area, buf);

        let title = Line::from(" Local Friend Code ").bold().yellow();
        let block = Block::new().borders(Borders::TOP).title(title);
        block.render(local_header_area, buf);
        let local_friend_code_line = Line::from(ctx.local_friend_code.to_pretty_string())
            .bold()
            .red()
            .centered();
        local_friend_code_line.render(local_code_area, buf);
    }

    fn render_error_area(
        &self,
        ctx: &AppContext,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let layout = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]);
        let [block_area, error_area] = layout.areas(area);

        let block = Block::new()
            .borders(Borders::BOTTOM)
            .border_type(BorderType::Plain);
        block.render(block_area, buf);

        let line = if let Some((e, _)) = &ctx.get_error() {
            Line::from(e.as_ref()).on_red().white()
        } else {
            Line::from("")
        };
        line.render(error_area, buf);
    }

    fn render_main_area(
        &self,
        ctx: &AppContext,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let layout = Layout::vertical([
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Min(1),
        ]);
        let [my_friend_code_area, line_area, actions_area] = layout.areas(area);

        self.render_friend_codes(ctx, my_friend_code_area, buf);

        let line_block = Block::bordered()
            .border_type(BorderType::Plain)
            .borders(Borders::BOTTOM);
        line_block.render(line_area, buf);

        let action_area_layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(2),
        ]);
        let [input_friend_code_area, _, error_area] = action_area_layout.areas(actions_area);

        CodeInput.render(ctx, input_friend_code_area, buf);

        self.render_error_area(ctx, error_area, buf);
    }

    fn render_call_area(
        &self,
        ctx: &AppContext,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let title = Line::from(" In Call ").bold().yellow();
        let block = Block::bordered()
            .border_type(BorderType::Plain)
            .title(title);
        let block_area = block.inner(area);
        block.render(area, buf);

        let addresses: Vec<String> = ctx
            .vchat
            .get_addresses()
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .to_vec()
            .into_iter()
            .map(|addr| addr.to_string())
            .collect();

        let list = List::new(addresses);
        list.render(block_area, buf);
    }
}

impl State for App {
    fn handle_event(
        &self,
        ctx: &mut crate::app::context::AppContext,
        event: &crossterm::event::Event,
    ) -> color_eyre::Result<()> {
        match event {
            event::Event::Key(key_event) => self.handle_key_event(ctx, &key_event)?,
            _ => {}
        };

        Ok(())
    }

    fn render(
        &self,
        ctx: &crate::app::context::AppContext,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
    ) {
        let title = Line::from(" VChat64 ").left_aligned().bold().yellow();
        let instructions = Line::from(vec![
            " Quit".into(),
            " <Q> ".bold().yellow(),
            " Input FC".into(),
            " <I> ".bold().yellow(),
        ])
        .left_aligned();
        let block = Block::bordered()
            .border_type(BorderType::Plain)
            .title(title)
            .title_bottom(instructions);
        let block_area = block.inner(area);
        block.render(area, buf);

        let horizontal_layout =
            Layout::horizontal([Constraint::Percentage(60), Constraint::Fill(1)]);
        let [main_area, call_area] = horizontal_layout.areas(block_area);

        self.render_main_area(ctx, main_area, buf);
        self.render_call_area(ctx, call_area, buf);
    }
}
