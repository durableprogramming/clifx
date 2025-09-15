use crossterm::{
    cursor, execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::size,
};
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

pub struct Shine2DConfig {
    pub base_color: (u8, u8, u8),
    pub speed: u64,
    pub easing: EasingFunction,
    pub duration: u64,
    pub cycles: u32,
    pub start: ShineStart,
    pub width: usize,
    pub blur: bool,
    pub padding: usize,
    pub shine_color: (u8, u8, u8),
    pub pause_length: Option<u64>,
    pub pause_position: f32,
    pub cycle_pre_delay: Option<u64>,
    pub cycle_post_delay: Option<u64>,
    pub cycle_switchback_delay: Option<u64>,
    pub opacity: f32,
    pub angle: f32,
    pub terminal_width: Option<usize>,
}

impl Default for Shine2DConfig {
    fn default() -> Self {
        Self {
            base_color: (255, 255, 255),
            speed: 50,
            easing: EasingFunction::Linear,
            duration: 2000,
            cycles: 1,
            start: ShineStart::Beginning,
            width: 3,
            blur: true,
            padding: 5,
            shine_color: (255, 255, 0),
            pause_length: None,
            pause_position: 0.5,
            cycle_pre_delay: None,
            cycle_post_delay: None,
            cycle_switchback_delay: None,
            opacity: 1.0,
            angle: 90.0, // Default to vertical shine
            terminal_width: None,
        }
    }
}

#[derive(Clone)]
pub enum ShineStart {
    Beginning,
    End,
}

#[derive(Clone)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl EasingFunction {
    fn apply(&self, t: f32) -> f32 {
        match self {
            EasingFunction::Linear => t,
            EasingFunction::EaseIn => t * t,
            EasingFunction::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            EasingFunction::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Position2D {
    x: usize,
    y: usize,
}

fn wrap_text_to_grid(text: &str, terminal_width: usize) -> Vec<Vec<char>> {
    let mut grid = Vec::new();
    let mut current_line = Vec::new();

    for ch in text.chars() {
        if ch == '\n' {
            grid.push(current_line);
            current_line = Vec::new();
        } else {
            current_line.push(ch);
            if current_line.len() >= terminal_width {
                grid.push(current_line);
                current_line = Vec::new();
            }
        }
    }

    if !current_line.is_empty() {
        grid.push(current_line);
    }

    grid
}

fn calculate_2d_shine_intensity(
    pos: &Position2D,
    shine_line: f32,
    angle: f32,
    width: f32,
    blur: bool,
) -> f32 {
    let angle_rad = angle.to_radians();
    let cos_angle = angle_rad.cos();
    let sin_angle = angle_rad.sin();

    // Calculate distance from point to shine line based on angle
    let distance = if angle.abs() < 0.01 {
        // Horizontal shine (angle ≈ 0)
        (pos.y as f32 - shine_line).abs()
    } else if (angle - 90.0).abs() < 0.01 {
        // Vertical shine (angle ≈ 90)
        (pos.x as f32 - shine_line).abs()
    } else {
        // Diagonal shine - distance from point to line
        // Line equation: cos(θ)x + sin(θ)y = shine_line
        let line_point_distance =
            (cos_angle * pos.x as f32 + sin_angle * pos.y as f32 - shine_line).abs();
        line_point_distance / (cos_angle * cos_angle + sin_angle * sin_angle).sqrt()
    };

    if distance <= width {
        if blur {
            1.0 - (distance / width)
        } else if distance <= 0.5 {
            1.0
        } else {
            0.0
        }
    } else {
        0.0
    }
}

pub fn apply_shine2d_effect(
    text: &str,
    config: &Shine2DConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();

    if text.is_empty() {
        println!();
        return Ok(());
    }

    let terminal_width = config
        .terminal_width
        .unwrap_or_else(|| size().map(|(w, _)| w as usize).unwrap_or(80));

    let grid = wrap_text_to_grid(text, terminal_width);
    let grid_height = grid.len();
    let max_width = grid.iter().map(|line| line.len()).max().unwrap_or(0);

    if grid_height == 0 || max_width == 0 {
        println!();
        return Ok(());
    }

    let frame_duration = Duration::from_millis(config.speed);
    let total_frames = (config.duration / config.speed) as usize;
    let cycles_to_run = if config.cycles == 0 {
        usize::MAX
    } else {
        config.cycles as usize
    };

    let base_color = Color::Rgb {
        r: config.base_color.0,
        g: config.base_color.1,
        b: config.base_color.2,
    };

    let shine_color = Color::Rgb {
        r: config.shine_color.0,
        g: config.shine_color.1,
        b: config.shine_color.2,
    };

    // Calculate the range for the shine to travel based on angle
    let diagonal_length = ((max_width * max_width + grid_height * grid_height) as f32).sqrt();
    let shine_range = diagonal_length + (2 * config.padding) as f32;

    execute!(stdout, cursor::SavePosition, cursor::Hide)?;

    for cycle in 0..cycles_to_run {
        if let Some(pre_delay) = config.cycle_pre_delay {
            thread::sleep(Duration::from_millis(pre_delay));
        }

        for frame in 0..total_frames {
            let progress = frame as f32 / (total_frames - 1) as f32;
            let eased_progress = config.easing.apply(progress);

            let prev_progress = if frame > 0 {
                let prev_frame_progress = (frame - 1) as f32 / (total_frames - 1) as f32;
                config.easing.apply(prev_frame_progress)
            } else {
                0.0
            };

            if let Some(switchback_delay) = config.cycle_switchback_delay {
                if frame > 0 && prev_progress < 0.5 && eased_progress >= 0.5 {
                    thread::sleep(Duration::from_millis(switchback_delay));
                }
            }

            let back_and_forth_progress = if eased_progress < 0.5 {
                eased_progress * 2.0
            } else {
                2.0 - (eased_progress * 2.0)
            };

            let shine_position = match config.start {
                ShineStart::Beginning => {
                    back_and_forth_progress * shine_range - config.padding as f32
                }
                ShineStart::End => {
                    (1.0 - back_and_forth_progress) * shine_range - config.padding as f32
                }
            };

            if let Some(pause_length) = config.pause_length {
                let normalized_position = (shine_position + config.padding as f32) / shine_range;
                let pause_tolerance = 0.05;

                if (normalized_position - config.pause_position).abs() < pause_tolerance {
                    thread::sleep(Duration::from_millis(pause_length));
                }
            }

            execute!(stdout, cursor::RestorePosition)?;

            for (y, line) in grid.iter().enumerate() {
                execute!(stdout, cursor::MoveToColumn(0))?;

                for (x, &ch) in line.iter().enumerate() {
                    let pos = Position2D { x, y };
                    let intensity = calculate_2d_shine_intensity(
                        &pos,
                        shine_position,
                        config.angle,
                        config.width as f32,
                        config.blur,
                    );

                    if intensity > 0.0 {
                        let opacity_adjusted_intensity = intensity * config.opacity;
                        let blended_color =
                            blend_colors(base_color, shine_color, opacity_adjusted_intensity);
                        execute!(stdout, SetForegroundColor(blended_color), Print(ch))?;
                    } else {
                        execute!(stdout, SetForegroundColor(base_color), Print(ch))?;
                    }
                }

                if y < grid.len() - 1 {
                    execute!(stdout, Print('\n'))?;
                }
            }

            execute!(stdout, ResetColor)?;
            stdout.flush()?;

            thread::sleep(frame_duration);
        }

        if let Some(post_delay) = config.cycle_post_delay {
            thread::sleep(Duration::from_millis(post_delay));
        }

        if config.cycles > 0 && cycle + 1 == cycles_to_run {
            break;
        }
    }

    execute!(stdout, cursor::Show)?;
    println!();
    Ok(())
}

fn blend_colors(base: Color, shine: Color, intensity: f32) -> Color {
    let intensity = intensity.clamp(0.0, 1.0);

    let (base_r, base_g, base_b) = match base {
        Color::Rgb { r, g, b } => (r, g, b),
        _ => (255, 255, 255),
    };

    let (shine_r, shine_g, shine_b) = match shine {
        Color::Rgb { r, g, b } => (r, g, b),
        _ => (255, 255, 255),
    };

    let blended_r = (base_r as f32 * (1.0 - intensity) + shine_r as f32 * intensity) as u8;
    let blended_g = (base_g as f32 * (1.0 - intensity) + shine_g as f32 * intensity) as u8;
    let blended_b = (base_b as f32 * (1.0 - intensity) + shine_b as f32 * intensity) as u8;

    Color::Rgb {
        r: blended_r,
        g: blended_g,
        b: blended_b,
    }
}
