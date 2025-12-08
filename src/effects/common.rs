use crossterm::style::Color;

#[derive(Debug, Clone)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl EasingFunction {
    pub fn apply(&self, t: f32) -> f32 {
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

pub fn blend_colors(base: Color, overlay: Color, intensity: f32) -> Color {
    let intensity = intensity.clamp(0.0, 1.0);

    let (base_r, base_g, base_b) = match base {
        Color::Rgb { r, g, b } => (r, g, b),
        _ => (255, 255, 255),
    };

    let (overlay_r, overlay_g, overlay_b) = match overlay {
        Color::Rgb { r, g, b } => (r, g, b),
        _ => (255, 255, 255),
    };

    let blended_r = (base_r as f32 * (1.0 - intensity) + overlay_r as f32 * intensity) as u8;
    let blended_g = (base_g as f32 * (1.0 - intensity) + overlay_g as f32 * intensity) as u8;
    let blended_b = (base_b as f32 * (1.0 - intensity) + overlay_b as f32 * intensity) as u8;

    Color::Rgb {
        r: blended_r,
        g: blended_g,
        b: blended_b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    const TEST_TOLERANCE: f32 = 0.001;

    #[test]
    fn test_easing_function_linear() {
        let easing = EasingFunction::Linear;

        assert_approx_eq!(easing.apply(0.0), 0.0, TEST_TOLERANCE);
        assert_approx_eq!(easing.apply(0.25), 0.25, TEST_TOLERANCE);
        assert_approx_eq!(easing.apply(0.5), 0.5, TEST_TOLERANCE);
        assert_approx_eq!(easing.apply(0.75), 0.75, TEST_TOLERANCE);
        assert_approx_eq!(easing.apply(1.0), 1.0, TEST_TOLERANCE);
    }

    #[test]
    fn test_easing_function_ease_in() {
        let easing = EasingFunction::EaseIn;

        assert_approx_eq!(easing.apply(0.0), 0.0, TEST_TOLERANCE);
        assert_approx_eq!(easing.apply(0.5), 0.25, TEST_TOLERANCE);
        assert_approx_eq!(easing.apply(1.0), 1.0, TEST_TOLERANCE);

        assert!(easing.apply(0.1) < 0.1);
        assert!(easing.apply(0.9) > 0.8);
    }

    #[test]
    fn test_easing_function_ease_out() {
        let easing = EasingFunction::EaseOut;

        assert_approx_eq!(easing.apply(0.0), 0.0, TEST_TOLERANCE);
        assert_approx_eq!(easing.apply(0.5), 0.75, TEST_TOLERANCE);
        assert_approx_eq!(easing.apply(1.0), 1.0, TEST_TOLERANCE);

        assert!(easing.apply(0.1) > 0.1);
        assert!(easing.apply(0.9) < 1.0);
    }

    #[test]
    fn test_easing_function_ease_in_out() {
        let easing = EasingFunction::EaseInOut;

        assert_approx_eq!(easing.apply(0.0), 0.0, TEST_TOLERANCE);
        assert_approx_eq!(easing.apply(0.5), 0.5, TEST_TOLERANCE);
        assert_approx_eq!(easing.apply(1.0), 1.0, TEST_TOLERANCE);

        let val_25 = easing.apply(0.25);
        let val_75 = easing.apply(0.75);
        assert_approx_eq!(val_25, 1.0 - val_75, 0.01);
    }

    #[test]
    fn test_easing_functions_range() {
        let functions = vec![
            EasingFunction::Linear,
            EasingFunction::EaseIn,
            EasingFunction::EaseOut,
            EasingFunction::EaseInOut,
        ];

        for easing in functions {
            assert_eq!(easing.apply(0.0), 0.0);
            assert_eq!(easing.apply(1.0), 1.0);

            let values: Vec<f32> = (0..=10).map(|i| easing.apply(i as f32 / 10.0)).collect();
            for i in 1..values.len() {
                assert!(
                    values[i] >= values[i - 1],
                    "Easing function should be monotonic increasing. {:?} at step {}: {} >= {}",
                    easing,
                    i,
                    values[i],
                    values[i - 1]
                );
            }
        }
    }

    #[test]
    fn test_blend_colors_basic() {
        let base = Color::Rgb {
            r: 100,
            g: 100,
            b: 100,
        };
        let overlay = Color::Rgb {
            r: 200,
            g: 200,
            b: 200,
        };

        if let Color::Rgb { r, g, b } = blend_colors(base, overlay, 0.0) {
            assert_eq!(r, 100);
            assert_eq!(g, 100);
            assert_eq!(b, 100);
        } else {
            panic!("Expected RGB color");
        }

        if let Color::Rgb { r, g, b } = blend_colors(base, overlay, 1.0) {
            assert_eq!(r, 200);
            assert_eq!(g, 200);
            assert_eq!(b, 200);
        } else {
            panic!("Expected RGB color");
        }
    }

    #[test]
    fn test_blend_colors_midpoint() {
        let base = Color::Rgb { r: 0, g: 0, b: 0 };
        let overlay = Color::Rgb {
            r: 255,
            g: 255,
            b: 255,
        };

        if let Color::Rgb { r, g, b } = blend_colors(base, overlay, 0.5) {
            assert_eq!(r, 127);
            assert_eq!(g, 127);
            assert_eq!(b, 127);
        } else {
            panic!("Expected RGB color");
        }
    }

    #[test]
    fn test_blend_colors_clamping() {
        let base = Color::Rgb {
            r: 100,
            g: 100,
            b: 100,
        };
        let overlay = Color::Rgb {
            r: 200,
            g: 200,
            b: 200,
        };

        let result_negative = blend_colors(base, overlay, -0.5);
        let result_over_one = blend_colors(base, overlay, 1.5);

        if let Color::Rgb { r, g, b } = result_negative {
            assert_eq!(r, 100);
            assert_eq!(g, 100);
            assert_eq!(b, 100);
        } else {
            panic!("Expected RGB color");
        }

        if let Color::Rgb { r, g, b } = result_over_one {
            assert_eq!(r, 200);
            assert_eq!(g, 200);
            assert_eq!(b, 200);
        } else {
            panic!("Expected RGB color");
        }
    }
}
