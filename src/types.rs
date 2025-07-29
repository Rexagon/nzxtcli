use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Color([u8; 3]);

impl Color {
    pub const BLACK: Self = Self::new(0, 0, 0);
    pub const WHITE: Self = Self::new(0xff, 0xff, 0xff);
    pub const RED: Self = Self::new(0xff, 0, 0);
    pub const GREEN: Self = Self::new(0, 0xff, 0);
    pub const BLUE: Self = Self::new(0, 0, 0xff);

    pub const fn wrap_slice(colors: &[Color]) -> &[u8] {
        let len = colors.len() * 3;
        // SAFETY: `Color` layout is the same as `[u8; 3]`.
        unsafe { std::slice::from_raw_parts(colors.as_ptr().cast(), len) }
    }

    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self([green, red, blue])
    }

    pub const fn red(&self) -> u8 {
        self.0[1]
    }

    pub const fn green(&self) -> u8 {
        self.0[0]
    }

    pub const fn blue(&self) -> u8 {
        self.0[2]
    }

    pub const fn inner(&self) -> &[u8; 3] {
        &self.0
    }

    pub const fn inner_mut(&mut self) -> &mut [u8; 3] {
        &mut self.0
    }
}

impl std::fmt::Display for Color {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{:02x}{:02x}{:02x}",
            self.red(),
            self.green(),
            self.blue()
        )
    }
}

impl FromStr for Color {
    type Err = anyhow::Error;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        s = match s.strip_prefix("#") {
            None => s,
            Some(s) => s,
        };

        if s.len() != 6 {
            anyhow::bail!("invalid color string length");
        }

        s = s.trim_start_matches("0");
        if s.is_empty() {
            s = "0";
        }

        let [_, r, g, b] = u32::from_str_radix(s, 16)?.to_be_bytes();
        Ok(Self([g, r, b]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_color() {
        let color = "000000".parse::<Color>().unwrap();
        assert_eq!(color, Color::BLACK);
        let color = "ff0000".parse::<Color>().unwrap();
        assert_eq!(color, Color::RED);

        for (str, expected) in [
            ("000000", Color::BLACK),
            ("ff0000", Color::RED),
            ("#00ff00", Color::GREEN),
            ("#0000ff", Color::BLUE),
            ("#200800", Color::new(32, 8, 0)),
        ] {
            let color = str.parse::<Color>().unwrap();
            assert_eq!(color, expected);
        }
    }
}
