pub const WIDE_VIEWPORT_MIN_WIDTH: f64 = 560.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportClass {
    Compact,
    Wide,
}

impl ViewportClass {
    pub fn from_width(width: f64) -> Self {
        if width >= WIDE_VIEWPORT_MIN_WIDTH {
            Self::Wide
        } else {
            Self::Compact
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_class_uses_the_focus_stacking_breakpoint() {
        assert_eq!(ViewportClass::from_width(375.0), ViewportClass::Compact);
        assert_eq!(ViewportClass::from_width(559.0), ViewportClass::Compact);
        assert_eq!(ViewportClass::from_width(560.0), ViewportClass::Wide);
    }
}
