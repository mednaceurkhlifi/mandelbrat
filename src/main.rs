use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    widgets::canvas::Canvas,
    Frame, Terminal,
};
use std::{
    io::{self, Result},
    time::Duration,
};

struct App {
    zoom: f64,
    center_x: f64,
    center_y: f64,
    max_iterations: u32,
}

impl Default for App {
    fn default() -> App {
        App {
            zoom: 1.0,
            center_x: -0.5,
            center_y: 0.0,
            max_iterations: 100,
        }
    }
}

impl App {
    fn zoom_in(&mut self) {
        self.zoom *= 1.5;
    }

    fn zoom_out(&mut self) {
        self.zoom /= 1.5;
    }

    fn move_left(&mut self) {
        self.center_x -= 0.1 / self.zoom;
    }

    fn move_right(&mut self) {
        self.center_x += 0.1 / self.zoom;
    }

    fn move_up(&mut self) {
        self.center_y -= 0.1 / self.zoom;
    }

    fn move_down(&mut self) {
        self.center_y += 0.1 / self.zoom;
    }

    fn increase_iterations(&mut self) {
        self.max_iterations = (self.max_iterations + 20).min(500);
    }

    fn decrease_iterations(&mut self) {
        self.max_iterations = (self.max_iterations.saturating_sub(20)).max(20);
    }
}

fn mandelbrot_iterations(c: (f64, f64), max_iter: u32) -> u32 {
    let mut z = (0.0, 0.0);
    for i in 0..max_iter {
        if z.0 * z.0 + z.1 * z.1 > 4.0 {
            return i;
        }
        z = (z.0 * z.0 - z.1 * z.1 + c.0, 2.0 * z.0 * z.1 + c.1);
    }
    max_iter
}

fn iteration_to_color(iterations: u32, max_iterations: u32) -> Color {
    if iterations == max_iterations {
        Color::Black
    } else {
        let ratio = iterations as f64 / max_iterations as f64;
        match (ratio * 8.0) as u32 {
            0 => Color::Blue,
            1 => Color::LightBlue,
            2 => Color::Cyan,
            3 => Color::Green,
            4 => Color::Yellow,
            5 => Color::LightRed,
            6 => Color::Red,
            7 => Color::Magenta,
            _ => Color::White,
        }
    }
}

fn main() -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::default();
    let res = run_app(&mut terminal, app);

    // restore terminal
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

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('+') | KeyCode::Char('=') => app.zoom_in(),
                    KeyCode::Char('-') => app.zoom_out(),
                    KeyCode::Left | KeyCode::Char('h') => app.move_left(),
                    KeyCode::Right | KeyCode::Char('l') => app.move_right(),
                    KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                    KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                    KeyCode::Char('i') => app.increase_iterations(),
                    KeyCode::Char('d') => app.decrease_iterations(),
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(f.area());

    let canvas = Canvas::default()
        .block(Block::default().borders(Borders::ALL).title("Mandelbrot Set"))
        .paint(|ctx| {
            let width = 80.0; 
            let height = 40.0; 
            
            
            let aspect_ratio = width / height;
            let range = 2.0 / app.zoom;
            let x_min = app.center_x - range * aspect_ratio;
            let x_max = app.center_x + range * aspect_ratio;
            let y_min = app.center_y - range;
            let y_max = app.center_y + range;

            for i in 0..80 {
                for j in 0..40 {
                    let x = x_min + (i as f64 / width) * (x_max - x_min);
                    let y = y_min + (j as f64 / height) * (y_max - y_min);
                    
                    let iterations = mandelbrot_iterations((x, y), app.max_iterations);
                    let color = iteration_to_color(iterations, app.max_iterations);
                    
                    ctx.print(
                        i as f64,
                        j as f64,
                        Span::styled("█", Style::default().fg(color))
                    );
                }
            }
        })
        .x_bounds([0.0, 80.0])
        .y_bounds([0.0, 40.0]);

    f.render_widget(canvas, chunks[0]);

    let info = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Controls: "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw("uit | "),
            Span::styled("+/-", Style::default().fg(Color::Yellow)),
            Span::raw(" zoom | "),
            Span::styled("arrows/hjkl", Style::default().fg(Color::Yellow)),
            Span::raw(" move | "),
            Span::styled("i/d", Style::default().fg(Color::Yellow)),
            Span::raw(" iterations"),
        ]),
        Line::from(vec![
            Span::raw(format!("Zoom: {:.2} | Center: ({:.4}, {:.4}) | Iterations: {}", 
                app.zoom, app.center_x, app.center_y, app.max_iterations)),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title("Info"));

    f.render_widget(info, chunks[1]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mandelbrot_iterations() {
        assert_eq!(mandelbrot_iterations((0.0, 0.0), 100), 100);
        
        assert!(mandelbrot_iterations((2.0, 2.0), 100) < 100);
    }

    #[test]
    fn test_color_mapping() {
        assert_eq!(iteration_to_color(100, 100), Color::Black);
        
        assert_eq!(iteration_to_color(0, 100), Color::Blue);
    }
}