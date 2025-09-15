use clap::{Parser, Subcommand, ValueEnum};
use std::io::{self, BufRead, BufReader};
use rand::Rng;

mod effects;
use effects::shine::{apply_shine_effect, ShineConfig, EasingFunction, ShineStart};
use effects::shine2d::{apply_shine2d_effect, Shine2DConfig};

#[derive(Parser)]
#[command(name = "clifx")]
#[command(about = "CLI effects for text processing")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(ValueEnum, Clone)]
pub enum EasingType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

#[derive(ValueEnum, Clone)]
pub enum StartType {
    Beginning,
    End,
}

#[derive(Subcommand)]
enum Commands {
    /// Apply shine effect to stdin
    Shine {
        /// Base color as RGB values (e.g., "255,255,0" for yellow)
        #[arg(long)]
        color: Option<String>,
        
        /// Animation speed in milliseconds between frames
        #[arg(long, default_value = "100")]
        speed: u64,
        
        /// Easing function for the shine animation
        #[arg(long, value_enum, default_value = "linear")]
        easing: EasingType,
        
        /// Duration of one complete cycle in milliseconds
        #[arg(long, default_value = "2000")]
        duration: u64,
        
        /// Number of complete back-and-forth cycles (0 for infinite)
        #[arg(long, default_value = "1")]
        cycles: u32,
        
        /// Starting direction of the shine effect
        #[arg(long, value_enum, default_value = "beginning")]
        start: StartType,
        
        /// Width of the shine effect in characters
        #[arg(long, default_value = "2")]
        width: usize,
        
        /// Enable blur effect for gradual highlighting
        #[arg(long, default_value = "true")]
        blur: bool,
        
        /// Padding to extend shine position past text boundaries
        #[arg(long, default_value = "5")]
        padding: usize,
        
        /// Shine color as RGB values (e.g., "255,255,255" for white)
        #[arg(long, default_value = "255,255,255")]
        shine_color: String,
        
        /// Length of pause in milliseconds (disabled if not specified)
        #[arg(long)]
        pause_length: Option<u64>,
        
        /// Position where shine pauses (0.0 to 1.0, where 0.5 is center)
        #[arg(long, default_value = "0.5")]
        pause_position: f32,
        
        /// Delay before each cycle starts in milliseconds (disabled if not specified)
        #[arg(long)]
        cycle_pre_delay: Option<u64>,
        
        /// Delay after each cycle completes in milliseconds (disabled if not specified)
        #[arg(long)]
        cycle_post_delay: Option<u64>,
        
        /// Delay when the shine changes direction (switchback) in milliseconds (disabled if not specified)
        #[arg(long)]
        cycle_switchback_delay: Option<u64>,
        
        /// Opacity of the shine effect (0.0 to 1.0, where 1.0 is full opacity)
        #[arg(long, default_value = "1.0")]
        opacity: f32,
    },
    /// Apply 2D shine effect to stdin with angle control and word wrapping
    Shine2d {
        /// Base color as RGB values (e.g., "255,255,0" for yellow)
        #[arg(long)]
        color: Option<String>,
        
        /// Animation speed in milliseconds between frames
        #[arg(long, default_value = "50")]
        speed: u64,
        
        /// Easing function for the shine animation
        #[arg(long, value_enum, default_value = "linear")]
        easing: EasingType,
        
        /// Duration of one complete cycle in milliseconds
        #[arg(long, default_value = "2000")]
        duration: u64,
        
        /// Number of complete back-and-forth cycles (0 for infinite)
        #[arg(long, default_value = "1")]
        cycles: u32,
        
        /// Starting direction of the shine effect
        #[arg(long, value_enum, default_value = "beginning")]
        start: StartType,
        
        /// Width of the shine effect in characters
        #[arg(long, default_value = "3")]
        width: usize,
        
        /// Enable blur effect for gradual highlighting
        #[arg(long, default_value = "true")]
        blur: bool,
        
        /// Padding to extend shine position past text boundaries
        #[arg(long, default_value = "5")]
        padding: usize,
        
        /// Shine color as RGB values (e.g., "255,255,0" for yellow)
        #[arg(long, default_value = "255,255,0")]
        shine_color: String,
        
        /// Length of pause in milliseconds (disabled if not specified)
        #[arg(long)]
        pause_length: Option<u64>,
        
        /// Position where shine pauses (0.0 to 1.0, where 0.5 is center)
        #[arg(long, default_value = "0.5")]
        pause_position: f32,
        
        /// Delay before each cycle starts in milliseconds (disabled if not specified)
        #[arg(long)]
        cycle_pre_delay: Option<u64>,
        
        /// Delay after each cycle completes in milliseconds (disabled if not specified)
        #[arg(long)]
        cycle_post_delay: Option<u64>,
        
        /// Delay when the shine changes direction (switchback) in milliseconds (disabled if not specified)
        #[arg(long)]
        cycle_switchback_delay: Option<u64>,
        
        /// Opacity of the shine effect (0.0 to 1.0, where 1.0 is full opacity)
        #[arg(long, default_value = "1.0")]
        opacity: f32,
        
        /// Angle of the shine line in degrees (0=horizontal, 90=vertical, 45=diagonal)
        #[arg(long, default_value = "90.0")]
        angle: f32,
        
        /// Terminal width for word wrapping (auto-detected if not specified)
        #[arg(long)]
        terminal_width: Option<usize>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Shine { color, speed, easing, duration, cycles, start, width, blur, padding, shine_color, pause_length, pause_position, cycle_pre_delay, cycle_post_delay, cycle_switchback_delay, opacity } => {
            let color_str = color.unwrap_or_else(generate_random_saturated_color);
            let rgb = parse_rgb_color(&color_str)?;
            let shine_rgb = parse_rgb_color(&shine_color)?;
            
            let easing_func = match easing {
                EasingType::Linear => EasingFunction::Linear,
                EasingType::EaseIn => EasingFunction::EaseIn,
                EasingType::EaseOut => EasingFunction::EaseOut,
                EasingType::EaseInOut => EasingFunction::EaseInOut,
            };
            
            let start_direction = match start {
                StartType::Beginning => ShineStart::Beginning,
                StartType::End => ShineStart::End,
            };
            
            let config = ShineConfig {
                base_color: rgb,
                speed,
                easing: easing_func,
                duration,
                cycles,
                start: start_direction,
                width,
                blur,
                padding,
                shine_color: shine_rgb,
                pause_length,
                pause_position: pause_position.clamp(0.0, 1.0),
                cycle_pre_delay,
                cycle_post_delay,
                cycle_switchback_delay,
                opacity: opacity.clamp(0.0, 1.0),
            };
            
            let stdin = io::stdin();
            let reader = BufReader::new(stdin.lock());
            
            for line in reader.lines() {
                let line = line?;
                apply_shine_effect(&line, &config)?;
            }
        },
        Commands::Shine2d { color, speed, easing, duration, cycles, start, width, blur, padding, shine_color, pause_length, pause_position, cycle_pre_delay, cycle_post_delay, cycle_switchback_delay, opacity, angle, terminal_width } => {
            use effects::shine2d::{EasingFunction as Shine2DEasingFunction, ShineStart as Shine2DShineStart};
            
            let color_str = color.unwrap_or_else(generate_random_saturated_color);
            let rgb = parse_rgb_color(&color_str)?;
            let shine_rgb = parse_rgb_color(&shine_color)?;
            
            let easing_func = match easing {
                EasingType::Linear => Shine2DEasingFunction::Linear,
                EasingType::EaseIn => Shine2DEasingFunction::EaseIn,
                EasingType::EaseOut => Shine2DEasingFunction::EaseOut,
                EasingType::EaseInOut => Shine2DEasingFunction::EaseInOut,
            };
            
            let start_direction = match start {
                StartType::Beginning => Shine2DShineStart::Beginning,
                StartType::End => Shine2DShineStart::End,
            };
            
            let config = Shine2DConfig {
                base_color: rgb,
                speed,
                easing: easing_func,
                duration,
                cycles,
                start: start_direction,
                width,
                blur,
                padding,
                shine_color: shine_rgb,
                pause_length,
                pause_position: pause_position.clamp(0.0, 1.0),
                cycle_pre_delay,
                cycle_post_delay,
                cycle_switchback_delay,
                opacity: opacity.clamp(0.0, 1.0),
                angle,
                terminal_width,
            };
            
            let stdin = io::stdin();
            let reader = BufReader::new(stdin.lock());
            let mut input_text = String::new();
            
            for line in reader.lines() {
                let line = line?;
                if !input_text.is_empty() {
                    input_text.push('\n');
                }
                input_text.push_str(&line);
            }
            
            apply_shine2d_effect(&input_text, &config)?;
        }
    }

    Ok(())
}

fn generate_random_saturated_color() -> String {
    let mut rng = rand::thread_rng();
    let hue = rng.gen_range(0.0..360.0);
    let saturation = 1.0; // Fully saturated
    let value = 1.0; // Full brightness
    
    // Convert HSV to RGB
    let c = value * saturation;
    let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0_f64).abs());
    let m = value - c;
    
    let (r_prime, g_prime, b_prime) = if hue < 60.0 {
        (c, x, 0.0)
    } else if hue < 120.0 {
        (x, c, 0.0)
    } else if hue < 180.0 {
        (0.0, c, x)
    } else if hue < 240.0 {
        (0.0, x, c)
    } else if hue < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    
    let r = ((r_prime + m) * 255.0) as u8;
    let g = ((g_prime + m) * 255.0) as u8;
    let b = ((b_prime + m) * 255.0) as u8;
    
    format!("{r},{g},{b}")
}

fn parse_rgb_color(color_str: &str) -> Result<(u8, u8, u8), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = color_str.split(',').collect();
    if parts.len() != 3 {
        return Err("Color must be in RGB format: r,g,b (e.g., 255,255,0)".into());
    }
    
    let r = parts[0].trim().parse::<u8>()?;
    let g = parts[1].trim().parse::<u8>()?;
    let b = parts[2].trim().parse::<u8>()?;
    
    Ok((r, g, b))
}
