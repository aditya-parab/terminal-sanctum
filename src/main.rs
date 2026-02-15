use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use ratatui::{Terminal, backend::CrosstermBackend};
use sanctum_core::{
    App, Avatar, InputMode, RaceType, Specialization, TICK_RATE, get_data_dir, load_avatar,
    save_avatar, ui,
};
use std::{
    io,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        println!("Terminal Sanctum {}", sanctum_core::APP_VERSION);
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(Avatar::summon());
    app.input_mode = InputMode::SelectingProfile;
    app.refresh_profiles();

    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;

    let res = run_app(&mut terminal, &mut app, rx, TICK_RATE);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("[System Error] Application terminated: {err:?}");
    }
    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    rx: Receiver<notify::Result<notify::Event>>,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        if !matches!(
            app.input_mode,
            InputMode::SelectingProfile
                | InputMode::CreatingProfile
                | InputMode::ConfirmingDelete(_)
        ) {
            app.avatar.update_tick();
            if app.avatar.needs_specialization()
                && !matches!(app.input_mode, InputMode::Specializing)
            {
                app.input_mode = InputMode::Specializing;
            }
        }

        terminal.draw(|f| ui(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match &app.input_mode {
                    InputMode::SelectingProfile => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('j') | KeyCode::Down => {
                            let i = match app.profile_list_state.selected() {
                                Some(i) => {
                                    if i >= app.profile_list.len().saturating_sub(1) {
                                        0
                                    } else {
                                        i + 1
                                    }
                                }
                                None => 0,
                            };
                            app.profile_list_state.select(Some(i));
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            let i = match app.profile_list_state.selected() {
                                Some(i) => {
                                    if i == 0 {
                                        app.profile_list.len().saturating_sub(1)
                                    } else {
                                        i - 1
                                    }
                                }
                                None => 0,
                            };
                            app.profile_list_state.select(Some(i));
                        }
                        KeyCode::Enter => {
                            if let Some(i) = app.profile_list_state.selected() {
                                if let Some(name) = app.profile_list.get(i) {
                                    if let Ok(avatar) = load_avatar(name) {
                                        app.avatar = avatar;
                                        let cwd = std::env::current_dir()
                                            .unwrap_or_else(|_| PathBuf::from("."));
                                        let abs_path = cwd
                                            .canonicalize()
                                            .unwrap_or(cwd)
                                            .to_string_lossy()
                                            .to_string();
                                        app.avatar.add_log(format!(
                                            "Soul Incarnated. Workspace: {abs_path}"
                                        ));
                                        app.avatar.contributions.clear();
                                        let history = Avatar::perform_passive_scan(".");
                                        for c in history {
                                            app.avatar.contributions.push(c);
                                        }
                                        app.avatar
                                            .contributions
                                            .sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                                        app.avatar.contributions.truncate(3);
                                        app.avatar.calculate_offline_gains();
                                        app.input_mode = InputMode::Normal;
                                    }
                                }
                            }
                        }
                        KeyCode::Char('c') => {
                            app.input_mode = InputMode::CreatingProfile;
                        }
                        KeyCode::Char('d') => {
                            if let Some(i) = app.profile_list_state.selected() {
                                if let Some(name) = app.profile_list.get(i) {
                                    app.input_mode = InputMode::ConfirmingDelete(name.clone());
                                }
                            }
                        }
                        _ => {}
                    },
                    InputMode::ConfirmingDelete(name) => match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            let name_to_del = name.clone();
                            delete_profile(&name_to_del).ok();
                            app.refresh_profiles();
                            app.input_mode = InputMode::SelectingProfile;
                        }
                        _ => app.input_mode = InputMode::SelectingProfile,
                    },
                    InputMode::CreatingProfile => match key.code {
                        KeyCode::Enter => {
                            let new_avatar = Avatar::summon();
                            save_avatar(&new_avatar).ok();
                            app.refresh_profiles();
                            app.input_mode = InputMode::SelectingProfile;
                        }
                        KeyCode::Esc => app.input_mode = InputMode::SelectingProfile,
                        _ => {}
                    },
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => {
                            save_avatar(&app.avatar).ok();
                            return Ok(());
                        }
                        KeyCode::Char('a') => {
                            app.input.clear();
                            app.input_mode = InputMode::Editing;
                        }
                        KeyCode::Char('e') => {
                            app.input_mode = InputMode::Emoting;
                        }
                        KeyCode::Char('u') => {
                            app.avatar.use_ultimate();
                            save_avatar(&app.avatar).ok();
                        }
                        KeyCode::Char('s') => {
                            save_avatar(&app.avatar).ok();
                            app.refresh_profiles();
                            app.input_mode = InputMode::SelectingProfile;
                        }
                        KeyCode::Char('j') | KeyCode::Down => app.next_task(),
                        KeyCode::Char('k') | KeyCode::Up => app.previous_task(),
                        KeyCode::Char(' ') => {
                            if let Some(i) = app.list_state.selected() {
                                app.avatar.toggle_task(i);
                                save_avatar(&app.avatar).ok();
                            }
                        }
                        KeyCode::Char('x') => {
                            if let Some(i) = app.list_state.selected() {
                                app.avatar.remove_task(i);
                                save_avatar(&app.avatar).ok();
                                if app.avatar.tasks.is_empty() {
                                    app.list_state.select(None);
                                } else {
                                    app.list_state.select(Some(i.saturating_sub(1)));
                                }
                            }
                        }
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            let desc = app.input.drain(..).collect::<String>();
                            if !desc.is_empty() {
                                app.avatar.add_task(desc);
                                app.list_state.select(Some(app.avatar.tasks.len() - 1));
                                save_avatar(&app.avatar).ok();
                            }
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char(c) => app.input.push(c),
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Esc => app.input_mode = InputMode::Normal,
                        _ => {}
                    },
                    InputMode::Specializing => match key.code {
                        KeyCode::Char('1') => {
                            let spec = match app.avatar.race_type {
                                RaceType::Orc => Specialization::Blademaster,
                                RaceType::Human => Specialization::Paladin,
                                RaceType::Undead => Specialization::DeathKnight,
                                RaceType::NightElf => Specialization::DemonHunter,
                            };
                            app.avatar.set_specialization(spec);
                            app.input_mode = InputMode::Normal;
                            save_avatar(&app.avatar).ok();
                        }
                        KeyCode::Char('2') => {
                            let spec = match app.avatar.race_type {
                                RaceType::Orc => Specialization::FarSeer,
                                RaceType::Human => Specialization::Archmage,
                                RaceType::Undead => Specialization::Lich,
                                RaceType::NightElf => Specialization::Druid,
                            };
                            app.avatar.set_specialization(spec);
                            app.input_mode = InputMode::Normal;
                            save_avatar(&app.avatar).ok();
                        }
                        _ => {}
                    },
                    InputMode::Emoting => match key.code {
                        KeyCode::Char('a') | KeyCode::Char('A') => {
                            app.avatar.trigger_emote(sanctum_core::Emote::Cheer);
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char('s') | KeyCode::Char('S') => {
                            app.avatar.trigger_emote(sanctum_core::Emote::Roar);
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char('d') | KeyCode::Char('D') => {
                            app.avatar.trigger_emote(sanctum_core::Emote::Dance);
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char('f') | KeyCode::Char('F') => {
                            app.avatar.trigger_emote(sanctum_core::Emote::Salute);
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char('g') | KeyCode::Char('G') => {
                            app.avatar.trigger_emote(sanctum_core::Emote::Ponder);
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char('h') | KeyCode::Char('H') => {
                            app.avatar.trigger_emote(sanctum_core::Emote::Flex);
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Esc => app.input_mode = InputMode::Normal,
                        _ => {}
                    },
                }
            }
        }

        while let Ok(Ok(event)) = rx.try_recv() {
            for path in event.paths {
                if let Some(path_str) = path.to_str() {
                    if path_str.contains(".git") && path_str.ends_with("logs/HEAD") {
                        let project_name = path
                            .ancestors()
                            .find(|p| p.ends_with(".git"))
                            .and_then(|p| p.parent())
                            .and_then(|p| p.file_name())
                            .and_then(|s| s.to_str())
                            .unwrap_or("Unknown Project")
                            .to_string();
                        if let Some(message) = Avatar::extract_last_commit_msg(&path) {
                            if !matches!(
                                app.input_mode,
                                InputMode::SelectingProfile
                                    | InputMode::CreatingProfile
                                    | InputMode::ConfirmingDelete(_)
                            ) {
                                app.avatar.link_contribution(project_name, message);
                                save_avatar(&app.avatar).ok();
                            }
                        }
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

pub fn delete_profile(name: &str) -> io::Result<()> {
    if let Some(mut path) = get_data_dir() {
        path.push(format!("{name}.json"));
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    } else {
        Err(io::Error::other("Could not resolve data directory"))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_binary_baseline() {
        assert_eq!(1 + 1, 2);
    }
}
