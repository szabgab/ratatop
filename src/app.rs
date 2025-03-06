use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols,
    text::Line,
    widgets::{Axis, Block, Chart, Dataset, GraphType, Paragraph, Row, Table},
    DefaultTerminal, Frame,
};
use sysinfo::ProcessesToUpdate;

#[derive(Debug, Default)]
pub struct App {
    /// Is the application running?
    running: bool,
    system: sysinfo::System,
    cpu: Vec<(f64, f64)>,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: true,
            system: sysinfo::System::new_all(),
            cpu: vec![],
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| {
                if frame.count() % 60 == 0 {
                    self.system.refresh_processes(ProcessesToUpdate::All, true);
                }
                self.system.refresh_cpu_all();
                self.cpu
                    .push((frame.count() as f64, self.system.global_cpu_usage() as f64));
                self.draw(frame)
            })?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/master/examples>
    fn draw(&mut self, frame: &mut Frame) {
        let [top, second, third] = Layout::vertical([
            Constraint::Percentage((25)),
            Constraint::Fill((1)),
            Constraint::Fill((1)),
        ])
        .areas(frame.area());

        let [left, right] =
            Layout::horizontal([Constraint::Percentage((50)), Constraint::Percentage((50))])
                .areas(second);

        let datasets = vec![
            // Scatter chart
            Dataset::default()
                .name("data1")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().cyan())
                .data(&self.cpu),
        ];
        let x_axis = Axis::default()
            .bounds([0f64, self.cpu.len() as f64])
            .style(Style::default().cyan());
        let y_axis = Axis::default()
            .bounds([0f64, 100f64])
            .style(Style::default().cyan());
        let chart = Chart::new(datasets)
            .block(Block::bordered().title("CPU"))
            .x_axis(x_axis)
            .y_axis(y_axis);

        frame.render_widget(Block::bordered(), left);
        frame.render_widget(Block::bordered(), right);

        frame.render_widget(chart, top);
        //frame.render_widget(Block::bordered(), second);
        self.render_processes(frame, third);
    }

    fn render_processes(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let mut rows: Vec<_> = vec![];
        for (pid, process) in self.system.processes() {
            let name = process.name().to_string_lossy().to_string();
            let cpu = process.cpu_usage();
            let row = vec![pid.to_string(), name, cpu.to_string()];
            rows.push(row);
        }

        rows.sort_by(|a, b| {
            let a = a[2].parse::<f32>().unwrap_or(0.0);
            let b = b[2].parse::<f32>().unwrap_or(0.0);
            b.partial_cmp(&a).unwrap()
        });

        let table = Table::new(
            rows.into_iter().map(Row::new).collect::<Vec<Row>>(),
            [
                Constraint::Max(10),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ],
        )
        .block(Block::bordered().title("Processes"))
        .header(Row::new(vec!["PID", "Name", "CPU"]).style(Style::default().bold()));

        frame.render_widget(table, area);
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        if event::poll(std::time::Duration::from_millis(16))? {
            match event::read()? {
                // it's important to check KeyEventKind::Press to avoid handling key release events
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
