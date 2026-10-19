use eframe::egui::Vec2;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardLevel {
    Samples,
    Compact,
    Summary,
    Detailed,
    Full,
}
impl CardLevel {
    pub const ALL: [Self; 5] = [
        Self::Samples,
        Self::Compact,
        Self::Summary,
        Self::Detailed,
        Self::Full,
    ];
    pub fn from_zoom(zoom: f32) -> Self {
        if zoom < 0.16 {
            Self::Samples
        } else if zoom < 0.38 {
            Self::Compact
        } else if zoom < 0.63 {
            Self::Summary
        } else if zoom < 0.88 {
            Self::Detailed
        } else {
            Self::Full
        }
    }
    pub fn zoom(self) -> f32 {
        match self {
            Self::Samples => 0.085,
            Self::Compact => 0.25,
            Self::Summary => 0.5,
            Self::Detailed => 0.75,
            Self::Full => 1.0,
        }
    }
    pub fn label(self) -> &'static str {
        if crate::i18n::is_english() {
            return match self {
                Self::Samples => "Minimal · swatches only (~5 mm)",
                Self::Compact => "Compact · swatches only",
                Self::Summary => "Summary · name and HEX",
                Self::Detailed => "Detailed · controls",
                Self::Full => "Full",
            };
        }
        match self {
            Self::Samples => "Mínimo · só amostras (~5 mm)",
            Self::Compact => "Compacto · só amostras",
            Self::Summary => "Resumo · nome e HEX",
            Self::Detailed => "Detalhado · controles",
            Self::Full => "Completo",
        }
    }
    pub fn short_label(self) -> &'static str {
        if crate::i18n::is_english() {
            return match self {
                Self::Samples => "Minimal",
                Self::Compact => "Compact",
                Self::Summary => "Summary",
                Self::Detailed => "Detailed",
                Self::Full => "Full",
            };
        }
        match self {
            Self::Samples => "Mínimo",
            Self::Compact => "Compacto",
            Self::Summary => "Resumo",
            Self::Detailed => "Detalhado",
            Self::Full => "Completo",
        }
    }
    pub fn size(self) -> Vec2 {
        match self {
            Self::Samples => Vec2::splat(96.0 * 5.0 / 25.4),
            Self::Compact => Vec2::splat(52.0),
            Self::Summary => Vec2::new(112.0, 108.0),
            Self::Detailed => Vec2::new(160.0, 166.0),
            Self::Full => Vec2::new(224.0, 210.0),
        }
    }
    pub fn gap(self) -> f32 {
        match self {
            Self::Samples => 5.0,
            Self::Compact => 7.0,
            Self::Summary => 10.0,
            _ => 14.0,
        }
    }
    pub fn swatch_only(self) -> bool {
        matches!(self, Self::Samples | Self::Compact)
    }
    pub fn controls(self) -> bool {
        matches!(self, Self::Detailed | Self::Full)
    }
}
