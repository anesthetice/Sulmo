use std::{
    path::PathBuf,
    io::{self,stdout}, time::{Instant, Duration}
};
use ratatui::{
    prelude::{CrosstermBackend, Backend, Layout, Direction, Constraint, Alignment, Margin},
    Terminal,
    Frame,
    widgets::{Block, Borders, Paragraph, Tabs, Wrap, Padding},
    style::{Style, Color, Modifier},
    text::{Span, Line}
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode},
};
use clipboard::{
    ClipboardProvider,
    ClipboardContext,
};

mod setup;
use setup::{
    load_ggml_models,
    load_llama_configuration,
    load_app_configuration,
};
mod configs;
use configs::{
    LlamaConfig,
    AppConfig,
};
mod utils;
use utils::{sleep, pathbuf_to_string};
mod conversation;
use conversation::Conversation;

const JANUARY_BLUE: Color = Color::Rgb(0, 161, 185);
const VIVID_MALACHITE: Color = Color::Rgb(0, 185, 24);

#[derive(PartialEq)]
enum Mode {
    Home,
    Chat,
    Settings,
    Exit,
}

impl Mode {
    pub const NUMBER_OF_MODES: usize = 3;
    pub fn from_usize(number: usize) -> Self {
        match number {
            0 => Self::Home,
            1 => Self::Chat,
            2 => Self::Settings,
            3 => Self::Exit,
            _ => Self::Home,
        }
    }
    pub fn to_usize(&self) -> usize {
        match self {
            Self::Home => 0,
            Self::Chat => 1,
            Self::Settings => 2,
            Self::Exit => 3,
        }
    }
}

struct Application {
    app_config: AppConfig,
    llama_config: LlamaConfig,
    mode: Mode,
    mode_index: usize,
    conversations: Vec<Conversation>,
    conversation_index: usize,
}

impl Application {
    pub fn new(app_config: AppConfig, llama_config: LlamaConfig, ggml_models: Vec<PathBuf>) -> Self {
        Self {
            app_config: app_config,
            llama_config: llama_config,
            mode: Mode::Home,
            mode_index: 0,
            conversations: ggml_models.into_iter().map(|model| {Conversation::new(model)}).collect(),
            conversation_index: 0,
        }
    }
    pub fn run<B: Backend>(mut self, terminal: &mut Terminal<B>) -> io::Result<()>{
        let mut last_tick = Instant::now();
        let tick_rate: Duration = Duration::from_millis(self.app_config.tick_rate);
        loop {
            terminal.draw(|frame| {self.ui(frame)})?;

            let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
            
            if event::poll(Duration::from_millis(10)).unwrap() {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char(chr) => {
                                if self.mode == Mode::Chat {
                                    self.conversations[self.conversation_index].push_char(chr);
                                }
                            }
                            KeyCode::Backspace => {
                                if self.mode == Mode::Chat {
                                    self.conversations[self.conversation_index].pop_back_input();
                                }
                            }
                            KeyCode::Esc => {
                                return Ok(());
                            },
                            KeyCode::Tab => {
                                self.next_mode();
                            },
                            KeyCode::PageUp => {
                                if self.mode == Mode::Chat {
                                    self.next_model();
                                }
                            },
                            KeyCode::PageDown => {
                                if self.mode == Mode::Chat {
                                    self.prev_model();
                                }
                            },
                            KeyCode::Enter => {
                                if self.mode == Mode::Exit {
                                    return Ok(());
                                } else if self.mode == Mode::Chat {
                                    self.conversations[self.conversation_index].run(&self.llama_config, &self.app_config);
                                }
                            },
                            KeyCode::End => {
                                if self.mode == Mode::Chat {
                                    self.conversations[self.conversation_index].reset_child();
                                }
                            },
                            KeyCode::Delete => {
                                if self.mode == Mode::Chat {
                                    self.conversations[self.conversation_index].pop_front();
                                }
                            },
                            KeyCode::Insert => {
                                let rctx = ClipboardContext::new();
                                if rctx.is_ok() {
                                    let mut ctx: ClipboardContext = rctx.unwrap();
                                    ctx.set_contents(self.conversations[self.conversation_index].get_pro_output().to_string());
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }
            if last_tick.elapsed() >= tick_rate {
                self.on_tick();
                last_tick = Instant::now();
            }
        }
    }
    pub fn ui<B:Backend>(&self, frame: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(frame.size());

        let tabs = Tabs::new(vec!["Home".to_string(), pathbuf_to_string(&self.conversations[self.conversation_index].model, 35, "?"), "Settings".to_string(), "Exit".to_string()])
            .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(VIVID_MALACHITE))
            .select(self.mode_index)
            .block(Block::new()
                .title(" Sulmo 0.0.1 ")
                .borders(Borders::all())
                .border_type(ratatui::widgets::BorderType::Rounded)
                .title_alignment(Alignment::Right)
                .style(Style::default().fg(JANUARY_BLUE))
        );
        frame.render_widget(tabs, chunks[0]);
        match self.mode {
            Mode::Home => {
                let paragraph = Paragraph::new("Welcome to Sulmo")
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(JANUARY_BLUE))
                    .wrap(Wrap { trim: true });
                let vertical_margin = {
                    let height = chunks[1].height;
                    if height % 2 == 1 {
                        (height-1)/2
                    } else {
                        if height != 0 {
                            (height/2)-1
                        } else {
                            0_u16
                        }
                    }
                };
                frame.render_widget(paragraph, chunks[1].inner(&Margin {vertical: vertical_margin, horizontal: 0}))
            },
            Mode::Chat => {
                let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
                .split(frame.size());

                let input_line = Line::from(self.conversations[self.conversation_index].get_usr_input());
                let input_paragraph = Paragraph::new(input_line)
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(JANUARY_BLUE))
                    .block(Block::new()
                        .borders(Borders::all())
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .style(Style::default().fg(JANUARY_BLUE))
                    );
                frame.render_widget(input_paragraph, chunks[2]);

                
                let mut lines = Vec::new();
                self.conversations[self.conversation_index].get_past_conversations_str().iter().for_each(|chunk| {
                    lines.push(Line::styled(chunk.0, Style::default().fg(JANUARY_BLUE)).alignment(Alignment::Right));
                    lines.push(Line::styled(chunk.1, Style::default().fg(VIVID_MALACHITE)).alignment(Alignment::Left));
                });
                if !self.conversations[self.conversation_index].get_pro_input().is_empty() {lines.push(Line::styled(self.conversations[self.conversation_index].get_pro_input(), Style::default().fg(JANUARY_BLUE).add_modifier(Modifier::BOLD)).alignment(Alignment::Right))};
                if !self.conversations[self.conversation_index].get_pro_output().is_empty() {lines.push(Line::styled(self.conversations[self.conversation_index].get_pro_output(), Style::default().fg(VIVID_MALACHITE).add_modifier(Modifier::BOLD)).alignment(Alignment::Left))};
                let output_paragraph = Paragraph::new(lines)
                    .alignment(Alignment::Center)
                    .block(Block::new()
                        .padding(Padding::new(4, 4, 1, 1))
                        .borders(Borders::all())
                        .border_type(ratatui::widgets::BorderType::Rounded)
                        .style(Style::default().fg(JANUARY_BLUE))
                    )
                .wrap(Wrap {trim: true});

                frame.render_widget(output_paragraph, chunks[1]);
            },
            Mode::Settings => {},
            Mode::Exit => {
                let text = Line::from(vec![
                    Span::styled("Press '", Style::default()),
                    Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("' in this window or '", Style::default()),
                    Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("' anywhere to exit the application", Style::default()),
                ]);
                let paragraph = Paragraph::new(text)
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(JANUARY_BLUE))
                    .wrap(Wrap { trim: true });
                let vertical_margin = {
                    let height = chunks[1].height;
                    if height % 2 == 1 {
                        (height-1)/2
                    } else {
                        if height != 0 {
                            (height/2)-1
                        } else {
                            0_u16
                        }
                    }
                };
                frame.render_widget(paragraph, chunks[1].inner(&Margin {vertical: vertical_margin, horizontal: 0}))
            }
        }
    }
    fn on_tick(&mut self) {
        self.conversations.iter_mut().for_each(|conv| {conv.check()});
    }
}

fn main() {
    // setup
    println!("         Loading configurations...");
    let app_config: AppConfig = load_app_configuration();
    let llama_config: LlamaConfig = load_llama_configuration();
    println!("         Loading ggml models...");
    let ggml_models: Vec<PathBuf> =  load_ggml_models();
    println!("         Setup complete, entering text user interface...\n\n\n");
    sleep(0.1);

    // text-user-interface
    let mut stdout = stdout();
    enable_raw_mode().unwrap();
    execute!(stdout, EnterAlternateScreen);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let application: Application = Application::new(app_config, llama_config, ggml_models);

    application.run(&mut terminal);

    execute!(terminal.backend_mut(), LeaveAlternateScreen);
    disable_raw_mode();
}

impl Application {
    fn next_mode(&mut self) {
        match self.mode {
            Mode::Home => {
                self.mode = Mode::Chat;
                self.mode_index = self.mode.to_usize();
            },
            Mode::Chat => {
                self.mode = Mode::Settings;
                self.mode_index = self.mode.to_usize();
            },
            Mode::Settings => {
                self.mode = Mode::Exit;
                self.mode_index = self.mode.to_usize();
            },
            Mode::Exit => {
                self.mode = Mode::Home;
                self.mode_index = self.mode.to_usize();
            },
        }
    }
    fn next_model(&mut self) {
        if self.conversation_index+1 < self.conversations.len() {
            self.conversation_index+=1;
        } else {
            self.conversation_index=0;
        }
    }
    fn prev_model(&mut self) {
        if self.conversation_index > 0 {
            self.conversation_index-=1;
        } else {
            self.conversation_index=self.conversations.len()-1
        }
    }
}

/*    

.block(Block::new()
                .title(" Sulmo 0.0.1 ")
                .borders(Borders::all())
                .border_type(ratatui::widgets::BorderType::Rounded)
                .title_alignment(Alignment::Center)
                .style(Style::default().fg(JANUARY_BLUE))
            );

    while child.try_wait().unwrap().is_none() {
        match child_stdout.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                let chunk = String::from_utf8_lossy(&buffer[..n]);
                output_string.push_str(&chunk);
            }
            Err(error) => panic!("{}", error),
        };
        stdout().write_all(output_string.as_bytes()); stdout().flush(); output_string.clear();
        thread::sleep(std::time::Duration::from_secs(1));
    }
    */