use std::time::Duration;

pub const MIN_DURATION_MS: u64 = 0;
pub const MAX_DURATION_MS: u64 = 1500;
pub const DEFAULT_DURATION_MS: u64 = 700;

pub const MIN_FPS: u32 = 5;
pub const MAX_FPS: u32 = 60;
pub const DEFAULT_FPS: u32 = 30;

const MAX_NAME_LEN: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationKind {
    Typewriter,
    Wave,
    Bounce,
}

impl AnimationKind {
    fn from_str_or_default(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "wave" => Self::Wave,
            "bounce" => Self::Bounce,
            // Unknown or empty names fall back to the default rather than erroring —
            // a typo in config.env must never break every new terminal.
            _ => Self::Typewriter,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Truecolor,
    Ansi256,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub name: String,
    pub animation: AnimationKind,
    pub duration: Duration,
    pub fps: u32,
    pub color_mode: ColorMode,
    pub debug: bool,
}

impl Config {
    /// Builds Config from an arbitrary (key, value) iterator instead of the real
    /// process environment, so tests never need to mutate global env state.
    pub fn from_env_iter<I>(vars: I) -> Self
    where
        I: IntoIterator<Item = (String, String)>,
    {
        let mut name = None;
        let mut user = None;
        let mut animation = None;
        let mut duration_ms = None;
        let mut fps = None;
        let mut colorterm = None;
        let mut debug = false;

        for (key, value) in vars {
            match key.as_str() {
                "TWP_NAME" => name = Some(value),
                "USER" => user = Some(value),
                "TWP_ANIMATION" => animation = Some(value),
                "TWP_DURATION_MS" => duration_ms = value.trim().parse::<u64>().ok(),
                "TWP_FPS" => fps = value.trim().parse::<u32>().ok(),
                "COLORTERM" => colorterm = Some(value),
                "TWP_DEBUG" => debug = value.trim() == "1",
                _ => {}
            }
        }

        let raw_name = name
            .filter(|s| !s.trim().is_empty())
            .or(user)
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "there".to_string());
        let name = truncate_name(raw_name.trim(), MAX_NAME_LEN);

        let animation = AnimationKind::from_str_or_default(animation.as_deref().unwrap_or(""));

        let duration_ms = duration_ms
            .unwrap_or(DEFAULT_DURATION_MS)
            .clamp(MIN_DURATION_MS, MAX_DURATION_MS);
        let fps = fps.unwrap_or(DEFAULT_FPS).clamp(MIN_FPS, MAX_FPS);

        let color_mode = match colorterm.as_deref() {
            Some("truecolor") | Some("24bit") => ColorMode::Truecolor,
            _ => ColorMode::Ansi256,
        };

        Config {
            name,
            animation,
            duration: Duration::from_millis(duration_ms),
            fps,
            color_mode,
            debug,
        }
    }

    pub fn from_process_env() -> Self {
        Self::from_env_iter(std::env::vars())
    }
}

/// Truncates on a char boundary so multi-byte UTF-8 (e.g. a Chinese name) never panics.
fn truncate_name(name: &str, max_len: usize) -> String {
    if name.chars().count() <= max_len {
        return name.to_string();
    }
    name.chars().take(max_len).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> Config {
        Config::from_env_iter(pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())))
    }

    #[test]
    fn defaults_when_nothing_set() {
        let c = env(&[]);
        assert_eq!(c.name, "there");
        assert_eq!(c.animation, AnimationKind::Typewriter);
        assert_eq!(c.duration, Duration::from_millis(DEFAULT_DURATION_MS));
        assert_eq!(c.fps, DEFAULT_FPS);
        assert_eq!(c.color_mode, ColorMode::Ansi256);
    }

    #[test]
    fn name_falls_back_to_user_then_there() {
        assert_eq!(env(&[("USER", "chai")]).name, "chai");
        assert_eq!(env(&[("TWP_NAME", "chai"), ("USER", "other")]).name, "chai");
        assert_eq!(env(&[("TWP_NAME", ""), ("USER", "")]).name, "there");
    }

    #[test]
    fn name_is_truncated_on_char_boundary() {
        let c = env(&[("TWP_NAME", "a-name-that-is-way-too-long")]);
        assert_eq!(c.name.chars().count(), MAX_NAME_LEN);

        // Non-ASCII (e.g. Chinese) must not panic when truncated.
        let c = env(&[("TWP_NAME", "柴由民柴由民柴由民柴由民柴由民柴由民柴由民")]);
        assert_eq!(c.name.chars().count(), MAX_NAME_LEN);
    }

    #[test]
    fn unknown_animation_falls_back_to_default() {
        assert_eq!(
            env(&[("TWP_ANIMATION", "not-a-real-one")]).animation,
            AnimationKind::Typewriter
        );
        assert_eq!(
            env(&[("TWP_ANIMATION", "wave")]).animation,
            AnimationKind::Wave
        );
        assert_eq!(
            env(&[("TWP_ANIMATION", "Bounce")]).animation,
            AnimationKind::Bounce
        );
    }

    #[test]
    fn duration_is_clamped() {
        assert_eq!(
            env(&[("TWP_DURATION_MS", "9999")]).duration,
            Duration::from_millis(MAX_DURATION_MS)
        );
        assert_eq!(
            env(&[("TWP_DURATION_MS", "not-a-number")]).duration,
            Duration::from_millis(DEFAULT_DURATION_MS)
        );
    }

    #[test]
    fn fps_is_clamped() {
        assert_eq!(env(&[("TWP_FPS", "0")]).fps, MIN_FPS);
        assert_eq!(env(&[("TWP_FPS", "999")]).fps, MAX_FPS);
    }

    #[test]
    fn colorterm_detects_truecolor() {
        assert_eq!(
            env(&[("COLORTERM", "truecolor")]).color_mode,
            ColorMode::Truecolor
        );
        assert_eq!(
            env(&[("COLORTERM", "24bit")]).color_mode,
            ColorMode::Truecolor
        );
        assert_eq!(env(&[]).color_mode, ColorMode::Ansi256);
    }

    #[test]
    fn debug_flag() {
        assert!(env(&[("TWP_DEBUG", "1")]).debug);
        assert!(!env(&[("TWP_DEBUG", "0")]).debug);
    }
}
