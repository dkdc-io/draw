// Stub module for the tiny-skia renderer (in progress)
// TODO: implement full renderer (#5)

/// Selection handle positions on a shape bounding box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HandlePosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Configuration for the renderer.
#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub scale: f64,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self { scale: 1.0 }
    }
}

/// Placeholder renderer (will use tiny-skia).
#[derive(Debug)]
pub struct Renderer {
    config: RenderConfig,
}

impl Renderer {
    pub fn new(config: RenderConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &RenderConfig {
        &self.config
    }
}
