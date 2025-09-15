use crossterm::{
    cursor, execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

pub struct ShineConfig {
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

pub fn apply_shine_effect(
    text: &str,
    config: &ShineConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();
    let text_chars: Vec<char> = text.chars().collect();
    let text_len = text_chars.len();

    if text_len == 0 {
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

    execute!(
        stdout,
        terminal::Clear(ClearType::CurrentLine),
        cursor::Hide
    )?;

    for cycle in 0..cycles_to_run {
        // Apply pre-cycle delay
        if let Some(pre_delay) = config.cycle_pre_delay {
            thread::sleep(Duration::from_millis(pre_delay));
        }

        for frame in 0..total_frames {
            let progress = frame as f32 / (total_frames - 1) as f32;
            let eased_progress = config.easing.apply(progress);

            // Check if we're at the switchback point (midpoint of cycle)
            let prev_progress = if frame > 0 {
                let prev_frame_progress = (frame - 1) as f32 / (total_frames - 1) as f32;
                config.easing.apply(prev_frame_progress)
            } else {
                0.0
            };

            // Apply switchback delay if we've crossed the midpoint (0.5)
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

            let total_range = text_len + (2 * config.padding);
            let shine_position = match config.start {
                ShineStart::Beginning => {
                    (back_and_forth_progress * (total_range as f32 - 1.0)) as isize
                        - config.padding as isize
                }
                ShineStart::End => {
                    ((1.0 - back_and_forth_progress) * (total_range as f32 - 1.0)) as isize
                        - config.padding as isize
                }
            };

            // Check if we should pause at the specified position
            if let Some(pause_length) = config.pause_length {
                let normalized_position =
                    (shine_position + config.padding as isize) as f32 / total_range as f32;
                let pause_tolerance = 0.05; // 5% tolerance for pause position

                if (normalized_position - config.pause_position).abs() < pause_tolerance {
                    thread::sleep(Duration::from_millis(pause_length));
                }
            }

            execute!(stdout, cursor::MoveToColumn(0))?;

            for (i, &ch) in text_chars.iter().enumerate() {
                let distance_from_shine = (i as isize - shine_position).abs() as f32;
                let shine_radius = config.width as f32;

                if distance_from_shine <= shine_radius {
                    let shine_intensity = if config.blur {
                        1.0 - (distance_from_shine / shine_radius)
                    } else if distance_from_shine == 0.0 {
                        1.0
                    } else {
                        0.0
                    };
                    // Apply opacity to the shine intensity
                    let opacity_adjusted_intensity = shine_intensity * config.opacity;
                    let blended_color =
                        blend_colors(base_color, shine_color, opacity_adjusted_intensity);
                    execute!(stdout, SetForegroundColor(blended_color), Print(ch))?;
                } else {
                    execute!(stdout, SetForegroundColor(base_color), Print(ch))?;
                }
            }

            execute!(stdout, ResetColor)?;
            stdout.flush()?;

            thread::sleep(frame_duration);
        }

        // Apply post-cycle delay
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
