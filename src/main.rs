use anyhow::{Context, Result};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ctrlc::set_handler;
use log::{error, info, warn};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph},
};
use rodio::{Decoder, OutputStream, Sink, Source};
use std::env;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use walkdir::WalkDir;

/// Supported audio file extensions
const SUPPORTED_EXTENSIONS: &[&str] = &["mp3", "wav", "ogg", "flac", "m4a"];

/// Global flag for graceful shutdown
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

/// Represents a music track with metadata
#[derive(Debug, Clone)]
pub struct Track {
    pub path: PathBuf,
    pub title: String,
    pub duration: Option<Duration>,
}

impl Track {
    pub fn new(path: PathBuf) -> Self {
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // Try to extract actual duration from MP3 file
        let duration = if let Some(ext) = path.extension() {
            if ext.to_string_lossy().to_lowercase() == "mp3" {
                match mp3_duration::from_path(&path) {
                    Ok(d) => {
                        info!(
                            "Extracted duration for '{}': {:.1}s",
                            title,
                            d.as_secs_f64()
                        );
                        Some(d)
                    }
                    Err(e) => {
                        warn!("Failed to extract duration for '{}': {}", title, e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        Self {
            path,
            title,
            duration,
        }
    }
}

/// Music player structure to manage playback state
pub struct MusicPlayer {
    tracks: Vec<Track>,
    current_index: usize,
    sink: Sink,
    _stream: OutputStream,
    is_paused: bool,
    is_shuffled: bool,
    repeat_mode: RepeatMode,
    start_time: Option<Instant>,
    elapsed_time: Duration, // Track actual playback time (excluding pauses)
}

#[derive(Clone, Copy, PartialEq)]
pub enum RepeatMode {
    None,
    One,
    All,
}

impl std::fmt::Display for RepeatMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepeatMode::None => write!(f, "Off"),
            RepeatMode::One => write!(f, "One"),
            RepeatMode::All => write!(f, "All"),
        }
    }
}

impl MusicPlayer {
    pub fn new() -> Result<Self> {
        let (_stream, stream_handle) =
            OutputStream::try_default().context("Failed to create audio output stream")?;

        let sink = Sink::try_new(&stream_handle).context("Failed to create audio sink")?;

        Ok(Self {
            tracks: Vec::new(),
            current_index: 0,
            sink,
            _stream,
            is_paused: false,
            is_shuffled: false,
            repeat_mode: RepeatMode::None,
            start_time: None,
            elapsed_time: Duration::default(),
        })
    }

    /// Load tracks from a directory or single file
    pub fn load_music<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        self.tracks.clear();

        if path.is_file() {
            if self.is_supported_audio_file(path) {
                self.tracks.push(Track::new(path.to_path_buf()));
                info!("Loaded single track: {}", path.display());
            } else {
                warn!("Unsupported file format: {}", path.display());
            }
        } else if path.is_dir() {
            let mut count = 0;
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() && self.is_supported_audio_file(entry.path()) {
                    self.tracks.push(Track::new(entry.path().to_path_buf()));
                    count += 1;
                }
            }
            info!("Loaded {} tracks from directory: {}", count, path.display());
        }

        if self.tracks.is_empty() {
            warn!("No supported audio files found in: {}", path.display());
        }

        Ok(())
    }

    /// Check if a file has a supported audio extension
    fn is_supported_audio_file<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
    }

    /// Find the next playable track starting from current index
    pub fn find_next_playable(&mut self) -> Option<usize> {
        if self.tracks.is_empty() {
            return None;
        }

        let start_index = self.current_index;
        let mut attempts = 0;

        loop {
            let track = &self.tracks[self.current_index];

            // Test if this track can be loaded
            if self.load_track(&track.path).is_ok() {
                return Some(self.current_index);
            }

            // Move to next track
            self.current_index = (self.current_index + 1) % self.tracks.len();
            attempts += 1;

            // If we've tried all tracks, give up
            if attempts >= self.tracks.len() || self.current_index == start_index {
                break;
            }
        }

        None
    }

    /// Play the current track
    pub fn play_current(&mut self) -> Result<()> {
        if self.tracks.is_empty() {
            warn!("No tracks loaded");
            return Ok(());
        }

        let track = &self.tracks[self.current_index];
        info!("Playing: {}", track.title);

        // Stop current playback
        self.sink.stop();

        // Load and play the new track
        match self.load_track(&track.path) {
            Ok(source) => {
                self.sink.append(source);
                self.sink.play();
                self.is_paused = false;
                self.start_time = Some(Instant::now());
                self.elapsed_time = Duration::default();
                Ok(())
            }
            Err(e) => {
                error!("Failed to play track '{}': {}", track.title, e);
                Ok(())
            }
        }
    }

    /// Load an audio track and return the decoded source
    fn load_track<P: AsRef<Path>>(&self, path: P) -> Result<Box<dyn Source<Item = f32> + Send>> {
        let file = File::open(&path)
            .with_context(|| format!("Failed to open audio file: {}", path.as_ref().display()))?;

        let reader = BufReader::new(file);
        let source = Decoder::new(reader)
            .with_context(|| format!("Failed to decode audio file: {}", path.as_ref().display()))?;

        Ok(Box::new(source.convert_samples()))
    }

    /// Move to the next track
    pub fn next_track(&mut self) -> Result<()> {
        if !self.tracks.is_empty() {
            self.current_index = (self.current_index + 1) % self.tracks.len();
            self.play_current()
        } else {
            Ok(())
        }
    }

    /// Move to the previous track
    pub fn previous_track(&mut self) -> Result<()> {
        if !self.tracks.is_empty() {
            self.current_index = if self.current_index == 0 {
                self.tracks.len() - 1
            } else {
                self.current_index - 1
            };
            self.play_current()
        } else {
            Ok(())
        }
    }

    /// Pause or resume playback
    pub fn toggle_pause(&mut self) {
        if self.sink.is_paused() {
            // Resuming - start a new timer
            self.sink.play();
            self.is_paused = false;
            self.start_time = Some(Instant::now());
        } else {
            // Pausing - add current session time to elapsed_time
            if let Some(start) = self.start_time {
                self.elapsed_time += start.elapsed();
            }
            self.sink.pause();
            self.is_paused = true;
            self.start_time = None;
        }
    }

    /// Stop playback
    pub fn stop(&mut self) {
        self.sink.stop();
        self.is_paused = false;
        self.start_time = None;
        self.elapsed_time = Duration::default();
        info!("Playback stopped");
    }

    /// Get current track info
    pub fn current_track(&self) -> Option<&Track> {
        self.tracks.get(self.current_index)
    }

    /// Get total number of tracks
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    /// Check if sink is empty (no audio playing)
    pub fn is_empty(&self) -> bool {
        self.is_track_finished()
    }

    /// Toggle shuffle mode
    pub fn toggle_shuffle(&mut self) {
        self.is_shuffled = !self.is_shuffled;
    }

    /// Cycle through repeat modes
    pub fn cycle_repeat(&mut self) {
        self.repeat_mode = match self.repeat_mode {
            RepeatMode::None => RepeatMode::One,
            RepeatMode::One => RepeatMode::All,
            RepeatMode::All => RepeatMode::None,
        };
    }

    /// Get current playback progress (0.0 to 1.0)
    pub fn get_progress(&self) -> f64 {
        // If the track is finished, return 100%
        if self.is_track_finished() {
            return 1.0;
        }

        // Calculate total elapsed time: previous elapsed + current session (if playing)
        let total_elapsed = if let Some(start) = self.start_time {
            // Currently playing - add current session time
            self.elapsed_time + start.elapsed()
        } else {
            // Paused or stopped - just use accumulated time
            self.elapsed_time
        };

        let elapsed_seconds = total_elapsed.as_secs() as f64;

        // Use actual track duration if available, otherwise fall back to estimate
        let duration_seconds = if let Some(current_track) = self.current_track() {
            if let Some(actual_duration) = current_track.duration {
                actual_duration.as_secs() as f64
            } else {
                300.0 // 5 minutes fallback for non-MP3 files
            }
        } else {
            300.0 // 5 minutes fallback if no current track
        };

        let progress = (elapsed_seconds / duration_seconds).min(1.0);

        // Log when track should be finishing
        if progress >= 0.95 {
            info!(
                "Track nearing completion: {:.1}% - elapsed: {:.1}s, duration: {:.1}s, sink_empty: {}",
                progress * 100.0,
                elapsed_seconds,
                duration_seconds,
                self.sink.empty()
            );
        }

        progress
    }

    /// Check if current track has finished playing
    pub fn is_track_finished(&self) -> bool {
        // First check if sink is empty (immediate detection)
        if self.sink.empty() {
            return true;
        }

        // Also check if elapsed time exceeds actual track duration
        if let Some(current_track) = self.current_track() {
            if let Some(actual_duration) = current_track.duration {
                let total_elapsed = if let Some(start) = self.start_time {
                    self.elapsed_time + start.elapsed()
                } else {
                    self.elapsed_time
                };

                // Consider finished if we've exceeded the track duration by a small margin
                return total_elapsed >= actual_duration + Duration::from_millis(500);
            }
        }

        false
    }
}

pub struct App {
    player: MusicPlayer,
    list_state: ListState,
    show_help: bool,
}

impl App {
    pub fn new(player: MusicPlayer) -> Self {
        let mut list_state = ListState::default();
        if !player.tracks.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            player,
            list_state,
            show_help: false,
        }
    }

    pub fn next_track(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.player.tracks.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.player.current_index = i;
    }

    pub fn previous_track(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.player.tracks.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.player.current_index = i;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
}

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    // Set up signal handler for graceful shutdown
    setup_signal_handlers();

    info!("Starting Terminal Music Player");

    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("🎵 Terminal Music Player");
        println!("Usage: {} <music_directory|music_file> [--test]", args[0]);
        println!("Example: {} ./music", args[0]);
        println!("Options:");
        println!("  --test    Exit immediately after testing playback (for testing)");
        println!("Supported formats: {}", SUPPORTED_EXTENSIONS.join(", "));
        return Ok(());
    }

    let path = PathBuf::from(&args[1]);
    let test_mode = args.len() > 2 && args[2] == "--test";

    // Create music player
    let mut player = MusicPlayer::new().context("Failed to initialize music player")?;

    // Load music
    player
        .load_music(&path)
        .with_context(|| format!("Failed to load music from: {}", path.display()))?;

    if player.track_count() == 0 {
        println!("❌ No supported audio files found in: {}", path.display());
        println!("Supported formats: {}", SUPPORTED_EXTENSIONS.join(", "));
        return Ok(());
    }

    // In test mode, just verify and exit
    if test_mode {
        println!("📁 Loaded {} tracks", player.track_count());
        if let Err(e) = player.play_current() {
            error!("Failed to start playback: {}", e);
            return Err(e);
        }
        println!("✅ Test mode: Playback started successfully");
        std::thread::sleep(Duration::from_millis(500));
        player.stop();
        return Ok(());
    }

    // Start the TUI
    run_tui(player)
}

fn run_tui(player: MusicPlayer) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(player);

    // Start playing the first track
    if !app.player.tracks.is_empty() {
        app.player.play_current()?;
    }

    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    // Stop playback
    app.player.stop();

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && handle_key_event(key, app).unwrap_or(false) {
                    return Ok(());
                }
            }
        }

        // Check if we should exit
        if SHUTDOWN.load(Ordering::Relaxed) {
            return Ok(());
        }

        // Auto-advance to next track if current one finished
        if app.player.is_empty() && !app.player.is_paused {
            let _ = app.player.next_track();
            // Sync the list selection with the new current track
            app.list_state.select(Some(app.player.current_index));
        }
    }
}

fn handle_key_event(key: KeyEvent, app: &mut App) -> Result<bool> {
    match key.code {
        // Quit
        KeyCode::Char('q') | KeyCode::Esc => return Ok(true),

        // Vim-style navigation (only moves selection, doesn't change playback)
        KeyCode::Char('j') | KeyCode::Down => {
            let i = match app.list_state.selected() {
                Some(i) => {
                    if i >= app.player.tracks.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            app.list_state.select(Some(i));
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let i = match app.list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        app.player.tracks.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            app.list_state.select(Some(i));
        }

        // Playback controls
        KeyCode::Char(' ') => {
            // If a different track is selected, play it. Otherwise, just pause/unpause
            if let Some(selected) = app.list_state.selected() {
                if selected != app.player.current_index {
                    app.player.current_index = selected;
                    app.player.play_current()?;
                } else {
                    app.player.toggle_pause();
                }
            } else {
                app.player.toggle_pause();
            }
        }
        KeyCode::Enter => {
            if let Some(selected) = app.list_state.selected() {
                app.player.current_index = selected;
                app.player.play_current()?;
            }
        }
        KeyCode::Char('n') => {
            app.player.next_track()?;
            // Sync the list selection with the current playing track
            app.list_state.select(Some(app.player.current_index));
        }
        KeyCode::Char('p') => {
            app.player.previous_track()?;
            // Sync the list selection with the current playing track
            app.list_state.select(Some(app.player.current_index));
        }

        // Advanced controls
        KeyCode::Char('s') => app.player.toggle_shuffle(),
        KeyCode::Char('r') => app.player.cycle_repeat(),
        KeyCode::Char('S') => app.player.stop(),

        // Help
        KeyCode::Char('?') | KeyCode::Char('h') => app.toggle_help(),

        _ => {}
    }

    Ok(false)
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // Track list
            Constraint::Length(3), // Currently playing
            Constraint::Length(3), // Progress bar
            Constraint::Length(3), // Controls
        ])
        .split(f.area());

    // Track list
    let items: Vec<ListItem> = app
        .player
        .tracks
        .iter()
        .enumerate()
        .map(|(i, track)| {
            let style = if i == app.player.current_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let prefix = if i == app.player.current_index {
                if app.player.is_paused { "⏸ " } else { "♪ " }
            } else {
                "  "
            };

            ListItem::new(format!("{}{}", prefix, track.title)).style(style)
        })
        .collect();

    let tracks = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "Tracks ({}/{})",
            app.list_state.selected().map(|i| i + 1).unwrap_or(1),
            app.player.tracks.len()
        )))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_stateful_widget(tracks, chunks[0], &mut app.list_state);

    // Currently playing
    let current_track = app
        .player
        .current_track()
        .map(|t| t.title.as_str())
        .unwrap_or("No track selected");

    let status = if app.player.is_paused {
        "⏸ Paused"
    } else {
        "♪ Playing"
    };

    let now_playing = Paragraph::new(format!("{}: {}", status, current_track))
        .block(Block::default().borders(Borders::ALL).title("Now Playing"))
        .alignment(Alignment::Center);

    f.render_widget(now_playing, chunks[1]);

    // Progress bar
    let progress = app.player.get_progress();
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Progress"))
        .gauge_style(Style::default().fg(Color::Green))
        .ratio(progress);

    f.render_widget(gauge, chunks[2]);

    // Controls info
    let controls_text = if app.show_help {
        vec![
            Line::from(vec![
                Span::raw("Navigation: "),
                Span::styled("j/↓", Style::default().fg(Color::Yellow)),
                Span::raw(" next, "),
                Span::styled("k/↑", Style::default().fg(Color::Yellow)),
                Span::raw(" prev"),
            ]),
            Line::from(vec![
                Span::raw("Playback: "),
                Span::styled("Space", Style::default().fg(Color::Yellow)),
                Span::raw(" play/pause, "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" play selected, "),
                Span::styled("n", Style::default().fg(Color::Yellow)),
                Span::raw(" next, "),
                Span::styled("p", Style::default().fg(Color::Yellow)),
                Span::raw(" prev"),
            ]),
            Line::from(vec![
                Span::raw("Other: "),
                Span::styled("s", Style::default().fg(Color::Yellow)),
                Span::raw(" shuffle, "),
                Span::styled("r", Style::default().fg(Color::Yellow)),
                Span::raw(" repeat, "),
                Span::styled("S", Style::default().fg(Color::Yellow)),
                Span::raw(" stop, "),
                Span::styled("q", Style::default().fg(Color::Yellow)),
                Span::raw(" quit, "),
                Span::styled("?", Style::default().fg(Color::Yellow)),
                Span::raw(" help"),
            ]),
        ]
    } else {
        vec![Line::from(vec![
            Span::raw("Shuffle: "),
            Span::styled(
                if app.player.is_shuffled { "On" } else { "Off" },
                if app.player.is_shuffled {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                },
            ),
            Span::raw(" | Repeat: "),
            Span::styled(
                app.player.repeat_mode.to_string(),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" | Press "),
            Span::styled("?", Style::default().fg(Color::Yellow)),
            Span::raw(" for help"),
        ])]
    };

    let controls = Paragraph::new(controls_text)
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .alignment(Alignment::Center);

    f.render_widget(controls, chunks[3]);

    // Show help overlay if requested
    if app.show_help {
        let help_area = centered_rect(60, 50, f.area());
        f.render_widget(Clear, help_area);

        let help_text = vec![
            Line::from("Vim-Inspired Music Player"),
            Line::from(""),
            Line::from("Navigation:"),
            Line::from("  j, ↓      - Move down in track list"),
            Line::from("  k, ↑      - Move up in track list"),
            Line::from(""),
            Line::from("Playback:"),
            Line::from("  Space     - Play selected track or pause/unpause"),
            Line::from("  Enter     - Play selected track"),
            Line::from("  n         - Next track"),
            Line::from("  p         - Previous track"),
            Line::from("  S         - Stop playback"),
            Line::from(""),
            Line::from("Modes:"),
            Line::from("  s         - Toggle shuffle"),
            Line::from("  r         - Cycle repeat mode (Off/One/All)"),
            Line::from(""),
            Line::from("Other:"),
            Line::from("  q, Esc    - Quit"),
            Line::from("  ?, h      - Toggle this help"),
            Line::from(""),
            Line::from("Press any key to close help..."),
        ];

        let help_popup = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("Help"))
            .alignment(Alignment::Left);

        f.render_widget(help_popup, help_area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn setup_signal_handlers() {
    // Set up Ctrl+C handler
    set_handler(move || {
        SHUTDOWN.store(true, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl+C handler");
}
