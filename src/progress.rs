use std::io::{self, stdout};
use std::time::{Duration, Instant};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
    cursor::{Hide, Show, MoveTo},
    event::{self, Event, KeyCode},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
    Terminal, backend::CrosstermBackend,
    text::{Line, Span},
};

pub struct CopyProgress {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    total_bytes: u64,
    current_bytes: u64,
    current_file: String,
    current_file_size: u64,
    current_file_progress: u64,
    last_update: Instant,  // 移除 start_time
    last_bytes: u64,
    last_speed: f64,
}

impl CopyProgress {
    pub fn new(total_bytes: u64) -> io::Result<Self> {
        let mut stdout = stdout();
        // 初始化时不需要预留空间，只需隐藏光标
        execute!(stdout, Hide)?;
        enable_raw_mode()?;
        
        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        
        let now = Instant::now();
        Ok(Self {
            terminal,
            total_bytes,
            current_bytes: 0,
            current_file: String::new(),
            current_file_size: 0,
            current_file_progress: 0,
            last_update: now,
            last_bytes: 0,
            last_speed: 0.0,
        })
    }

    fn calculate_speed(&self) -> f64 {
        let elapsed = self.last_update.elapsed().as_secs_f64();
        if elapsed < 0.1 {
            return self.last_speed;
        }

        let bytes_per_sec = (self.current_bytes - self.last_bytes) as f64 / elapsed;
        let speed = bytes_per_sec / (1024.0 * 1024.0);

        // 使用更平滑的移动平均
        if self.last_speed > 0.0 {
            self.last_speed * 0.8 + speed * 0.2
        } else {
            speed
        }
    }

    pub fn set_current_file(&mut self, file_name: &str, file_size: u64) {
        self.current_file = file_name.to_string();
        self.current_file_size = file_size;
        self.current_file_progress = 0;
        self.redraw().unwrap();
    }

    pub fn inc_current(&mut self, delta: u64) {
        self.current_bytes += delta;
        self.current_file_progress += delta;

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();

        // 更频繁地更新速度
        if elapsed >= 0.1 {
            let bytes_per_sec = (self.current_bytes - self.last_bytes) as f64 / elapsed;
            let speed = bytes_per_sec / (1024.0 * 1024.0);

            // 更平滑的速度更新
            self.last_speed = if self.last_speed > 0.0 {
                self.last_speed * 0.8 + speed * 0.2
            } else {
                speed
            };

            self.last_update = now;
            self.last_bytes = self.current_bytes;
        }

        self.redraw().unwrap();
    }

    fn redraw(&mut self) -> io::Result<()> {
        // 检查 Ctrl+C
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('c') && key.modifiers.contains(event::KeyModifiers::CONTROL) {
                    self.finish()?;
                    std::process::exit(0);
                }
            }
        }

        // 预先计算所有需要的值
        let total_bytes = self.total_bytes;
        let current_bytes = self.current_bytes;
        let current_file = self.current_file.clone();
        let current_file_size = self.current_file_size;
        let current_file_progress = self.current_file_progress;
        let speed = self.calculate_speed();

        let total_progress = (current_bytes as f64 / total_bytes as f64 * 100.0) as u16;
        let current_progress = (current_file_progress as f64 / current_file_size.max(1) as f64 * 100.0) as u16;

        let calculate_inner_rect = |rect: Rect| -> Rect {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1)])
                .margin(1)
                .split(rect)[0]
        };

        self.terminal.draw(|f| {
            let display_area = Rect {
                x: 0,
                y: 0,
                width: f.size().width,
                height: 7,  // 减少高度到最小需求
            };

            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),  // 总进度
                    Constraint::Length(1),  // 总进度详情
                    Constraint::Length(3),  // 当前文件
                ])
                .split(display_area);

            // 渲染总进度
            let total_block = Block::default()
                .title("Total Progress")
                .borders(Borders::ALL);
            f.render_widget(total_block, main_layout[0]);

            let gauge = Gauge::default()
                .block(Block::default())
                .gauge_style(Style::default().fg(Color::Cyan))
                .percent(total_progress)
                .label(format!("{}%", total_progress));
            f.render_widget(gauge, calculate_inner_rect(main_layout[0]));

            // 渲染进度详情和速度在同一行
            let details = format!(
                "{:.2} MiB / {:.2} MiB    Speed: {:.2} MiB/s",
                current_bytes as f64 / 1024.0 / 1024.0,
                total_bytes as f64 / 1024.0 / 1024.0,
                speed
            );
            let total_detail = Paragraph::new(Line::from(vec![
                Span::raw(details)
            ]));
            f.render_widget(total_detail, main_layout[1]);

            // 渲染当前文件
            let current_block = Block::default()
                .title(format!("Current File: {}", current_file))
                .borders(Borders::ALL);
            f.render_widget(current_block, main_layout[2]);

            let current_gauge = Gauge::default()
                .block(Block::default())
                .gauge_style(Style::default().fg(Color::Cyan))
                .percent(current_progress)
                .label(format!("{}%", current_progress));
            f.render_widget(current_gauge, calculate_inner_rect(main_layout[2]));
        })?;

        Ok(())
    }

    pub fn finish(&mut self) -> io::Result<()> {
        execute!(
            self.terminal.backend_mut(),
            Show,
            MoveTo(0, 7)
        )?;
        disable_raw_mode()?;
        Ok(())
    }
}

impl Drop for CopyProgress {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}