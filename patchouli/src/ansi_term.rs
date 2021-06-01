use core::u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TermColour {
    Black = 0,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl TermColour {
    pub fn as_fg(self) -> u8 {
        self as u8 + 30
    }
    pub fn as_bg(self) -> u8 {
        self as u8 + 40
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TermGMode {
    Reset = 0,
    Bold,
    Dim,
    Italic,
    Underline,
    Blinking,
    Inverse,
    Invisible,
    StrikeThrough,
}
impl TermGMode {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TermStyle {
    pub mode: Option<TermGMode>,
    pub fg: Option<TermColour>,
    pub bg: Option<TermColour>,
}

impl TermStyle {
    pub const fn new(
        mode: Option<TermGMode>,
        fg: Option<TermColour>,
        bg: Option<TermColour>,
    ) -> Self {
        TermStyle { mode, fg, bg }
    }
    pub const fn fg(fg: TermColour) -> Self {
        TermStyle::new(None, Some(fg), None)
    }
    pub const fn with_bg(mut self, bg: TermColour) -> Self {
        self.bg = Some(bg);
        self
    }
    pub const fn with_mode(mut self, mode: TermGMode) -> Self {
        self.mode = Some(mode);
        self
    }
    pub const RESET: Self = TermStyle::new(None, None, None);
}
impl Default for TermStyle {
    fn default() -> Self {
        Self::RESET
    }
}
