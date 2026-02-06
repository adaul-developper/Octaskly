use crate::protocol::WorkerInfo;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame, Terminal,
};
use std::io;

// Terminal UI tab types
// Jenis tab UI terminal
pub enum Tab {
    Workers,
    Tasks,
    Logs,
}

pub struct Dashboard {
    current_tab: usize,
    tabs: Vec<&'static str>,
    logs: Vec<String>,
    workers_display: Vec<String>,
    tasks_display: Vec<String>,
}

impl Dashboard {
    pub fn new() -> Self {
        Self {
            current_tab: 0,
            tabs: vec!["Workers", "Tasks", "Logs"],
            logs: Vec::new(),
            workers_display: Vec::new(),
            tasks_display: Vec::new(),
        }
    }

    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % self.tabs.len();
    }

    pub fn prev_tab(&mut self) {
        if self.current_tab > 0 {
            self.current_tab -= 1;
        } else {
            self.current_tab = self.tabs.len() - 1;
        }
    }

    pub fn add_log(&mut self, message: String) {
        self.logs.push(message);
        // Keep only last 100 logs
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }

    pub fn update_workers(&mut self, workers: Vec<WorkerInfo>) {
        self.workers_display.clear();
        for worker in workers {
            let status = if worker.is_idle() { "[IDLE]" } else { "[BUSY]" };
            let line = format!(
                "{:20} | {} | Jobs: {}/{} | {}",
                worker.name, status, worker.current_jobs, worker.max_jobs, worker.platform
            );
            self.workers_display.push(line);
        }
    }

    pub fn update_tasks(&mut self, completed: usize, queued: usize) {
        self.tasks_display.clear();
        self.tasks_display.push(format!("Completed Tasks: {}", completed));
        self.tasks_display.push(format!("Queued Tasks: {}", queued));
    }

    pub fn draw(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(3), Constraint::Min(10)])
            .split(f.area());

        // Draw tabs
        // Gambar tab
        let tab_titles: Vec<&str> = self.tabs.iter().copied().collect();
        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::BOTTOM).title("OCTASKLY Dashboard"))
            .select(self.current_tab)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            );
        f.render_widget(tabs, chunks[0]);

        // Draw content based on selected tab
        match self.current_tab {
            0 => self.draw_workers_tab(f, chunks[1]),
            1 => self.draw_tasks_tab(f, chunks[1]),
            2 => self.draw_logs_tab(f, chunks[1]),
            _ => {}
        }
    }

    fn draw_workers_tab(&self, f: &mut Frame, area: Rect) {
        if self.workers_display.is_empty() {
            let empty_msg = Paragraph::new("No workers connected")
                .block(Block::default().borders(Borders::ALL).title("Workers"));
            f.render_widget(empty_msg, area);
            return;
        }

        let items: Vec<ListItem> = self
            .workers_display
            .iter()
            .map(|w| ListItem::new(w.clone()))
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Workers"))
            .style(Style::default().fg(Color::White));

        f.render_widget(list, area);
    }

    fn draw_tasks_tab(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .tasks_display
            .iter()
            .map(|t| ListItem::new(t.clone()))
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Tasks"))
            .style(Style::default().fg(Color::White));

        f.render_widget(list, area);
    }

    fn draw_logs_tab(&self, f: &mut Frame, area: Rect) {
        let lines: Vec<Line> = self
            .logs
            .iter()
            .rev()
            .take(20)
            .map(|log| Line::raw(log.clone()))
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Logs"));

        f.render_widget(paragraph, area);
    }
}

impl Default for Dashboard {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Ui {
    dashboard: Dashboard,
    terminal: Option<Terminal<CrosstermBackend<io::Stdout>>>,
}

impl Ui {
    pub fn new() -> io::Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        
        Ok(Self {
            dashboard: Dashboard::new(),
            terminal: Some(terminal),
        })
    }

    pub fn refresh(&mut self) -> io::Result<()> {
        if let Some(terminal) = &mut self.terminal {
            terminal.draw(|f| {
                self.dashboard.draw(f);
            })?;
        }
        Ok(())
    }

    pub fn next_tab(&mut self) {
        self.dashboard.next_tab();
    }

    pub fn prev_tab(&mut self) {
        self.dashboard.prev_tab();
    }

    pub fn add_log(&mut self, message: String) {
        self.dashboard.add_log(message);
    }

    pub fn update_workers(&mut self, workers: Vec<WorkerInfo>) {
        self.dashboard.update_workers(workers);
    }

    pub fn update_tasks(&mut self, completed: usize, queued: usize) {
        self.dashboard.update_tasks(completed, queued);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_new() {
        let dashboard = Dashboard::new();
        assert_eq!(dashboard.current_tab, 0);
        assert_eq!(dashboard.tabs.len(), 3);
    }

    #[test]
    fn test_dashboard_tab_navigation() {
        let mut dashboard = Dashboard::new();
        dashboard.next_tab();
        assert_eq!(dashboard.current_tab, 1);
        dashboard.prev_tab();
        assert_eq!(dashboard.current_tab, 0);
    }

    #[test]
    fn test_dashboard_logs() {
        let mut dashboard = Dashboard::new();
        dashboard.add_log("Test log".to_string());
        assert!(!dashboard.logs.is_empty());
    }
}

