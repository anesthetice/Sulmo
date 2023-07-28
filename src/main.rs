use std::{
    path::PathBuf,
    process::{Command, Stdio},
    io::{self, stdout, Read, Write},
};
use ratatui::{
    prelude::{CrosstermBackend, Backend, Layout, Direction, Constraint, Alignment, Margin},
    Terminal,
    Frame,
    widgets::{ListItem, List, Block, Borders, Paragraph, Tabs, Wrap},
    style::{Style, Color, Modifier},
    text::{Span, Line}
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode},
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
    mode: Mode,
    mode_index: usize,
    ggml_models: Vec<PathBuf>,
    model_index: usize,
}

impl Application {
    pub fn new(ggml_models: Vec<PathBuf>) -> Self {
        Self {
            mode: Mode::Home,
            mode_index: 0,
            ggml_models: ggml_models,
            model_index: 0,
        }
    }
    pub fn run<B: Backend>(mut self, terminal: &mut Terminal<B>, ) -> io::Result<()>{
        loop {
            terminal.draw(|frame| {self.ui(frame)})?;
            
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
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
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
        
    }
    pub fn ui<B:Backend>(&self, frame: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(frame.size());

        let tabs = Tabs::new(vec!["Home".to_string(), pathbuf_to_string(&self.ggml_models[self.model_index], 35, "?"), "Settings".to_string(), "Exit".to_string()])
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
                let paragraph = Paragraph::new("Welcome to Sulmo, a terminal application designed to be a stylish yet barebones way of using llama.cpp to generate text")
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
            Mode::Chat => {},
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

    let mut application: Application = Application::new(ggml_models);

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
        if self.model_index+1 < self.ggml_models.len() {
            self.model_index+=1;
        } else {
            self.model_index=0;
        }
    }
    fn prev_model(&mut self) {
        if self.model_index > 0 {
            self.model_index-=1;
        } else {
            self.model_index=self.ggml_models.len()-1
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

    // choosing a model
    for (index, filepath) in ggml_models.iter().enumerate() {
        println!("{}. {}", index, filepath.file_name().unwrap_or("?".as_ref()).to_str().unwrap_or("?"))
    }
    let mut chosen_model: PathBuf = PathBuf::new();
    let mut user_input: String = String::new();
    while !chosen_model.exists() {
        stdout().write_all("Please select a model: ".as_bytes()); stdout().flush(); io::stdin().read_line(&mut user_input).expect("failed to read user input"); print!("\n");
        match user_input.trim().parse::<usize>() {
            Ok(num) => {
                if num < ggml_models.len() {
                    chosen_model = ggml_models[num].clone();
                }
            },
            Err(..) => (),
        }
        user_input.clear()
    }

    // choosing a prompt
    user_input.clear();
    stdout().write_all("Prompt:".as_bytes()); stdout().flush(); io::stdin().read_line(&mut user_input).expect("failed to read user input"); print!("\n");
    let prompt: String = format!("###Instruction: {} \\n###Response: ", user_input.trim());
    drop(user_input);

    let mut args: Vec<String> = llama_config.to_args();
    args.push("--model".to_string()); args.push(chosen_model.to_str().unwrap().to_string());
    args.push("--prompt".to_string()); args.push(prompt.to_string());

    let mut child = Command::new("llama-cpp/main")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn()
        .expect("failed to execute llama-cpp/main");

    let mut child_stdout = child.stdout.take().unwrap();

    let mut output_string: String = String::new();
    let mut buffer: [u8; 1024] = [0; 1024];
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