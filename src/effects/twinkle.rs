use crossterm::{
    cursor, execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use rand::Rng;
use std::collections::HashMap;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

pub struct TwinkleConfig {
    pub base_color: (u8, u8, u8),
    pub twinkle_color: (u8, u8, u8),
    pub speed: u64,
    pub easing: EasingFunction,
    pub duration: u64,
    pub cycles: u32,
    pub twinkle_ratio: Option<f32>,
    pub min_twinkle_count: Option<usize>,
    pub max_twinkle_count: Option<usize>,
    pub twinkling_percentage: f32,
    pub star_mode: bool,
}

impl Default for TwinkleConfig {
    fn default() -> Self {
        Self {
            base_color: (255, 255, 255),
            twinkle_color: (255, 255, 0),
            speed: 100,
            easing: EasingFunction::Linear,
            duration: 3000,
            cycles: 1,
            twinkle_ratio: Some(0.3),
            min_twinkle_count: None,
            max_twinkle_count: None,
            twinkling_percentage: 0.8,
            star_mode: false,
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Clone)]
struct TwinkleState {
    phase: f32,
    duration: f32,
    pause_duration: f32,
}

const TWINKLE_CHARS: &[char] = &['.', '·', '•', '⋅', '∘', '○', '●'];
const TWINKLE_CHARS_STAR: &[char] = &['.', '✦', '✧', '⋆', '✩', '✪', '✫', '⭐', '*'];

fn calculate_three_phase_progress(
    phase: f32,
    pause_duration_ratio: f32,
    easing: &EasingFunction,
) -> f32 {
    let phase = phase.clamp(0.0, 1.0);

    // Total animation divided into three parts:
    // - ease_up_duration: from 0 to max intensity
    // - pause_duration: stay at max intensity
    // - ease_down_duration: from max intensity back to 0
    let ease_duration = (1.0 - pause_duration_ratio) / 2.0; // Split remaining time equally for ease up and down
    let ease_up_end = ease_duration;
    let pause_end = ease_up_end + pause_duration_ratio;

    if phase <= ease_up_end {
        // Ease up phase
        let local_progress = phase / ease_duration;
        easing.apply(local_progress)
    } else if phase <= pause_end {
        // Pause phase - stay at full intensity
        1.0
    } else {
        // Ease down phase
        let local_progress = (phase - pause_end) / ease_duration;
        let reverse_progress = 1.0 - local_progress;
        easing.apply(reverse_progress)
    }
}

fn get_twinkle_char(progress: f32, star_mode: bool) -> char {
    let eased_progress = progress.clamp(0.0, 1.0);
    let chars = if star_mode {
        TWINKLE_CHARS_STAR
    } else {
        TWINKLE_CHARS
    };
    let index = (eased_progress * (chars.len() - 1) as f32).round() as usize;
    chars[index.min(chars.len() - 1)]
}

fn blend_colors(base: Color, twinkle: Color, intensity: f32) -> Color {
    let intensity = intensity.clamp(0.0, 1.0);

    let (base_r, base_g, base_b) = match base {
        Color::Rgb { r, g, b } => (r, g, b),
        _ => (255, 255, 255),
    };

    let (twinkle_r, twinkle_g, twinkle_b) = match twinkle {
        Color::Rgb { r, g, b } => (r, g, b),
        _ => (255, 255, 255),
    };

    let blended_r = (base_r as f32 * (1.0 - intensity) + twinkle_r as f32 * intensity) as u8;
    let blended_g = (base_g as f32 * (1.0 - intensity) + twinkle_g as f32 * intensity) as u8;
    let blended_b = (base_b as f32 * (1.0 - intensity) + twinkle_b as f32 * intensity) as u8;

    Color::Rgb {
        r: blended_r,
        g: blended_g,
        b: blended_b,
    }
}

pub fn apply_twinkle_effect(
    text: &str,
    config: &TwinkleConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();
    let text_chars: Vec<char> = text.chars().collect();
    let text_len = text_chars.len();

    if text_len == 0 {
        println!();
        return Ok(());
    }

    // Find all period positions
    let period_positions: Vec<usize> = text_chars
        .iter()
        .enumerate()
        .filter_map(|(i, &ch)| if ch == '.' { Some(i) } else { None })
        .collect();

    if period_positions.is_empty() {
        // No periods to twinkle, just print the text normally
        let base_color = Color::Rgb {
            r: config.base_color.0,
            g: config.base_color.1,
            b: config.base_color.2,
        };
        execute!(
            stdout,
            SetForegroundColor(base_color),
            Print(text),
            ResetColor
        )?;
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

    let twinkle_color = Color::Rgb {
        r: config.twinkle_color.0,
        g: config.twinkle_color.1,
        b: config.twinkle_color.2,
    };

    let mut rng = rand::thread_rng();
    let mut twinkle_states: HashMap<usize, TwinkleState> = HashMap::new();

    execute!(
        stdout,
        terminal::Clear(ClearType::CurrentLine),
        cursor::Hide
    )?;

    for cycle in 0..cycles_to_run {
        for _frame in 0..total_frames {
            // Determine if twinkling should be active this frame
            let should_twinkle = rng.gen::<f32>() < config.twinkling_percentage;

            if should_twinkle {
                // Calculate how many periods should be twinkling
                let twinkle_count = if let (Some(min), Some(max)) =
                    (config.min_twinkle_count, config.max_twinkle_count)
                {
                    rng.gen_range(min..=max.min(period_positions.len()))
                } else if let Some(ratio) = config.twinkle_ratio {
                    ((period_positions.len() as f32 * ratio).round() as usize).max(1)
                } else if let Some(min) = config.min_twinkle_count {
                    min.min(period_positions.len())
                } else if let Some(max) = config.max_twinkle_count {
                    max.min(period_positions.len())
                } else {
                    (period_positions.len() as f32 * 0.3).round() as usize
                };

                // Update existing twinkle states
                twinkle_states.retain(|_, state| {
                    state.phase += 1.0 / state.duration;
                    state.phase <= 1.0
                });

                // Add new twinkles if we need more
                let current_twinkles = twinkle_states.len();
                if current_twinkles < twinkle_count {
                    let available_positions: Vec<usize> = period_positions
                        .iter()
                        .filter(|&&pos| !twinkle_states.contains_key(&pos))
                        .copied()
                        .collect();

                    let new_twinkles_needed = twinkle_count - current_twinkles;
                    for _ in 0..new_twinkles_needed {
                        if !available_positions.is_empty() {
                            let pos =
                                available_positions[rng.gen_range(0..available_positions.len())];
                            let duration = rng.gen_range(20.0..60.0); // Random duration between 20-60 frames
                            let pause_duration = rng.gen_range(0.1..0.2); // 10-20% of total duration as pause
                            twinkle_states.insert(
                                pos,
                                TwinkleState {
                                    phase: 0.0,
                                    duration,
                                    pause_duration,
                                },
                            );
                        }
                    }
                }
            }

            execute!(stdout, cursor::MoveToColumn(0))?;

            for (i, &ch) in text_chars.iter().enumerate() {
                if let Some(state) = twinkle_states.get(&i) {
                    let eased_progress = calculate_three_phase_progress(
                        state.phase,
                        state.pause_duration,
                        &config.easing,
                    );
                    let twinkle_char = get_twinkle_char(eased_progress, config.star_mode);
                    let color_intensity = eased_progress;
                    let blended_color = blend_colors(base_color, twinkle_color, color_intensity);
                    execute!(
                        stdout,
                        SetForegroundColor(blended_color),
                        Print(twinkle_char)
                    )?;
                } else {
                    execute!(stdout, SetForegroundColor(base_color), Print(ch))?;
                }
            }

            execute!(stdout, ResetColor)?;
            stdout.flush()?;

            thread::sleep(frame_duration);
        }

        if config.cycles > 0 && cycle + 1 == cycles_to_run {
            break;
        }
    }

    execute!(stdout, cursor::Show)?;
    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    const TEST_TOLERANCE: f32 = 0.001;

    #[test]
    fn test_twinkle_config_default() {
        let config = TwinkleConfig::default();

        assert_eq!(config.base_color, (255, 255, 255));
        assert_eq!(config.twinkle_color, (255, 255, 0));
        assert_eq!(config.speed, 100);
        assert_eq!(config.duration, 3000);
        assert_eq!(config.cycles, 1);
        assert_eq!(config.twinkle_ratio, Some(0.3));
        assert_eq!(config.min_twinkle_count, None);
        assert_eq!(config.max_twinkle_count, None);
        assert_eq!(config.twinkling_percentage, 0.8);
        assert!(!config.star_mode);
    }

    #[test]
    fn test_twinkle_config_creation() {
        let config = TwinkleConfig {
            base_color: (255, 0, 0),
            twinkle_color: (0, 255, 0),
            speed: 50,
            easing: EasingFunction::Linear,
            duration: 1000,
            cycles: 2,
            twinkle_ratio: Some(0.5),
            min_twinkle_count: Some(1),
            max_twinkle_count: Some(5),
            twinkling_percentage: 0.9,
            star_mode: true,
        };

        assert_eq!(config.base_color, (255, 0, 0));
        assert_eq!(config.twinkle_color, (0, 255, 0));
        assert_eq!(config.speed, 50);
        assert_eq!(config.duration, 1000);
        assert_eq!(config.cycles, 2);
        assert_eq!(config.twinkle_ratio, Some(0.5));
        assert_eq!(config.min_twinkle_count, Some(1));
        assert_eq!(config.max_twinkle_count, Some(5));
        assert_eq!(config.twinkling_percentage, 0.9);
        assert!(config.star_mode);
    }

    #[test]
    fn test_easing_function_twinkle_linear() {
        let easing = EasingFunction::Linear;

        assert_approx_eq!(easing.apply(0.0), 0.0, TEST_TOLERANCE);
        assert_approx_eq!(easing.apply(0.25), 0.25, TEST_TOLERANCE);
        assert_approx_eq!(easing.apply(0.5), 0.5, TEST_TOLERANCE);
        assert_approx_eq!(easing.apply(0.75), 0.75, TEST_TOLERANCE);
        assert_approx_eq!(easing.apply(1.0), 1.0, TEST_TOLERANCE);
    }

    #[test]
    fn test_easing_function_twinkle_ease_in() {
        let easing = EasingFunction::EaseIn;

        assert_approx_eq!(easing.apply(0.0), 0.0, TEST_TOLERANCE);
        assert_approx_eq!(easing.apply(0.5), 0.25, TEST_TOLERANCE);
        assert_approx_eq!(easing.apply(1.0), 1.0, TEST_TOLERANCE);

        // Ease-in should start slow and accelerate
        assert!(easing.apply(0.1) < 0.1);
        assert!(easing.apply(0.9) > 0.8);
    }

    #[test]
    fn test_easing_function_twinkle_ease_out() {
        let easing = EasingFunction::EaseOut;

        assert_approx_eq!(easing.apply(0.0), 0.0, TEST_TOLERANCE);
        assert_approx_eq!(easing.apply(0.5), 0.75, TEST_TOLERANCE);
        assert_approx_eq!(easing.apply(1.0), 1.0, TEST_TOLERANCE);

        // Ease-out should start fast and decelerate
        assert!(easing.apply(0.1) > 0.1);
        assert!(easing.apply(0.9) < 1.0);
    }

    #[test]
    fn test_easing_function_twinkle_ease_in_out() {
        let easing = EasingFunction::EaseInOut;

        assert_approx_eq!(easing.apply(0.0), 0.0, TEST_TOLERANCE);
        assert_approx_eq!(easing.apply(0.5), 0.5, TEST_TOLERANCE);
        assert_approx_eq!(easing.apply(1.0), 1.0, TEST_TOLERANCE);

        // Ease-in-out should be symmetric around 0.5
        let val_25 = easing.apply(0.25);
        let val_75 = easing.apply(0.75);
        assert_approx_eq!(val_25, 1.0 - val_75, 0.01);
    }

    #[test]
    fn test_easing_functions_monotonic() {
        let functions = vec![
            EasingFunction::Linear,
            EasingFunction::EaseIn,
            EasingFunction::EaseOut,
            EasingFunction::EaseInOut,
        ];

        for easing in functions {
            // Test edge cases
            assert_eq!(easing.apply(0.0), 0.0);
            assert_eq!(easing.apply(1.0), 1.0);

            // Test monotonic increasing property
            let values: Vec<f32> = (0..=10).map(|i| easing.apply(i as f32 / 10.0)).collect();
            for i in 1..values.len() {
                assert!(
                    values[i] >= values[i - 1],
                    "Easing function should be monotonic increasing at step {}: {} >= {}",
                    i,
                    values[i],
                    values[i - 1]
                );
            }
        }
    }

    #[test]
    fn test_twinkle_ratio_clamping() {
        // Test ratio values within valid range
        let valid_ratios = [0.0, 0.3, 0.5, 1.0];
        for &ratio in &valid_ratios {
            let config = TwinkleConfig {
                twinkle_ratio: Some(ratio),
                ..TwinkleConfig::default()
            };
            assert_eq!(config.twinkle_ratio, Some(ratio));
        }
    }

    #[test]
    fn test_twinkling_percentage_valid_range() {
        // Test percentage values within valid range
        let valid_percentages = [0.0, 0.5, 0.8, 1.0];
        for &percentage in &valid_percentages {
            let config = TwinkleConfig {
                twinkling_percentage: percentage,
                ..TwinkleConfig::default()
            };
            assert_eq!(config.twinkling_percentage, percentage);
        }
    }

    #[test]
    fn test_min_max_twinkle_counts() {
        let config = TwinkleConfig {
            min_twinkle_count: Some(2),
            max_twinkle_count: Some(10),
            ..TwinkleConfig::default()
        };

        assert_eq!(config.min_twinkle_count, Some(2));
        assert_eq!(config.max_twinkle_count, Some(10));
    }

    #[test]
    fn test_star_mode_toggle() {
        let config_no_star = TwinkleConfig {
            star_mode: false,
            ..TwinkleConfig::default()
        };
        assert!(!config_no_star.star_mode);

        let config_star = TwinkleConfig {
            star_mode: true,
            ..TwinkleConfig::default()
        };
        assert!(config_star.star_mode);
    }
}
