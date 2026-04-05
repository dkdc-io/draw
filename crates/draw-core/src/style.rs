use serde::{Deserialize, Serialize};

// ── Defaults ─────────────────────────────────────────────────────────

pub const DEFAULT_STROKE_COLOR: &str = "#e2e8f0";
pub const DEFAULT_STROKE_WIDTH: f64 = 2.0;
pub const DEFAULT_FILL_COLOR: &str = "#3b82f6";
pub const DEFAULT_FONT_FAMILY: &str = "Inter, sans-serif";
pub const DEFAULT_FONT_SIZE: f64 = 20.0;

// ── StrokeStyle ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrokeStyle {
    pub color: String,
    pub width: f64,
    #[serde(default)]
    pub dash: Vec<f64>,
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self {
            color: DEFAULT_STROKE_COLOR.to_string(),
            width: DEFAULT_STROKE_WIDTH,
            dash: vec![],
        }
    }
}

// ── FillStyle ────────────────────────────────────────────────────────

pub const DEFAULT_HACHURE_GAP: f64 = 10.0;
pub const DEFAULT_HACHURE_ANGLE: f64 = -0.785; // -45 degrees in radians

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FillStyle {
    pub color: String,
    pub style: FillType,
    #[serde(default = "default_hachure_gap")]
    pub gap: f64,
    #[serde(default = "default_hachure_angle")]
    pub angle: f64,
}

impl Default for FillStyle {
    fn default() -> Self {
        Self {
            color: DEFAULT_FILL_COLOR.to_string(),
            style: FillType::Hachure,
            gap: DEFAULT_HACHURE_GAP,
            angle: DEFAULT_HACHURE_ANGLE,
        }
    }
}

fn default_hachure_gap() -> f64 {
    DEFAULT_HACHURE_GAP
}
fn default_hachure_angle() -> f64 {
    DEFAULT_HACHURE_ANGLE
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FillType {
    Solid,
    Hachure,
    CrossHatch,
    None,
}

// ── FontStyle ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontStyle {
    pub family: String,
    pub size: f64,
    pub align: TextAlign,
}

impl Default for FontStyle {
    fn default() -> Self {
        Self {
            family: DEFAULT_FONT_FAMILY.to_string(),
            size: DEFAULT_FONT_SIZE,
            align: TextAlign::Left,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

// ── Arrowhead ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Arrowhead {
    Arrow,
    Triangle,
    Dot,
}
