use crate::{
    Background, Div, Window, colors::DefaultAppearance, div, hsla, linear_color_stop,
    linear_gradient, opaque_grey, px, styled::Styled,
};

/// A semantic role for a material surface.
///
/// Roles describe the job a surface performs in the interface rather than the
/// exact platform-specific effect used to render it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaterialRole {
    /// A primary navigation/sidebar surface.
    Sidebar,
    /// A top toolbar or header surface.
    Toolbar,
    /// A secondary inspector surface.
    Inspector,
    /// A floating overlay, sheet, or popover-like surface.
    Overlay,
    /// A search field or search-field container surface.
    SearchField,
    /// A surface used to separate floating controls from scrolling content.
    ScrollEdge,
    /// A background extension area behind a floating sidebar or toolbar.
    BackgroundExtension,
    /// A grouped set of controls that should read as one elevated cluster.
    GroupedControls,
}

/// A rendering variant for a semantic material surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MaterialVariant {
    /// A balanced adaptive material tuned by appearance and activation state.
    #[default]
    Adaptive,
    /// A more transparent variant intended for media-rich surroundings.
    Clear,
    /// A more opaque fallback for platforms or states where translucency should
    /// be reduced.
    OpaqueFallback,
}

/// A relative emphasis level for a material surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MaterialEmphasis {
    /// A subdued surface that should not dominate nearby content.
    Low,
    /// A balanced surface for common chrome containers.
    #[default]
    Medium,
    /// A stronger surface for the most important floating or navigational
    /// chrome.
    High,
}

/// A semantic style definition for a material surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaterialStyle {
    /// The purpose of the surface in the UI.
    pub role: MaterialRole,
    /// The rendering variant to prefer for the surface.
    pub variant: MaterialVariant,
    /// The visual prominence of the surface relative to nearby chrome.
    pub emphasis: MaterialEmphasis,
    /// Whether the surface should use a softly tinted material treatment.
    pub tinted: bool,
    /// Whether the surface represents a directly interactive control cluster.
    pub interactive: bool,
}

impl MaterialStyle {
    /// Create a new material style for the given role.
    pub fn new(role: MaterialRole) -> Self {
        Self {
            role,
            variant: MaterialVariant::Adaptive,
            emphasis: MaterialEmphasis::Medium,
            tinted: false,
            interactive: false,
        }
    }

    /// Create a sidebar material style.
    pub fn sidebar() -> Self {
        Self::new(MaterialRole::Sidebar)
    }

    /// Create a toolbar material style.
    pub fn toolbar() -> Self {
        Self::new(MaterialRole::Toolbar)
    }

    /// Create an inspector material style.
    pub fn inspector() -> Self {
        Self::new(MaterialRole::Inspector)
    }

    /// Create an overlay material style.
    pub fn overlay() -> Self {
        Self::new(MaterialRole::Overlay)
    }

    /// Create a search field material style.
    pub fn search_field() -> Self {
        Self::new(MaterialRole::SearchField)
    }

    /// Create a scroll-edge material style.
    pub fn scroll_edge() -> Self {
        Self::new(MaterialRole::ScrollEdge)
    }

    /// Create a background-extension material style.
    pub fn background_extension() -> Self {
        Self::new(MaterialRole::BackgroundExtension)
    }

    /// Create a grouped-controls material style.
    pub fn grouped_controls() -> Self {
        Self::new(MaterialRole::GroupedControls)
    }

    /// Override the rendering variant.
    pub fn variant(mut self, variant: MaterialVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Override the emphasis level.
    pub fn emphasis(mut self, emphasis: MaterialEmphasis) -> Self {
        self.emphasis = emphasis;
        self
    }

    /// Enable a softly tinted treatment.
    pub fn tinted(mut self, tinted: bool) -> Self {
        self.tinted = tinted;
        self
    }

    /// Mark the surface as representing an interactive control cluster.
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }
}

/// Platform capabilities currently exposed by GPUI's material system.
///
/// This reports what the GPUI layer natively supports today. The initial
/// semantic material system starts with a cross-platform GPUI-rendered fallback,
/// so native platform-specific capabilities are conservative by default and can
/// expand over time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PlatformMaterialCapabilities {
    /// Whether native glass components are supported directly by GPUI.
    pub native_glass: bool,
    /// Whether background-extension behavior is supported directly by GPUI.
    pub background_extension: bool,
    /// Whether native scroll-edge material effects are supported directly by GPUI.
    pub scroll_edge_effects: bool,
    /// Whether concentric layout-region behaviors are supported directly by GPUI.
    pub concentric_layout_regions: bool,
    /// Whether adaptive grouped-control chrome is supported directly by GPUI.
    pub adaptive_grouped_controls: bool,
}

/// Return the native material capabilities currently implemented by GPUI.
pub fn platform_material_capabilities() -> PlatformMaterialCapabilities {
    PlatformMaterialCapabilities::default()
}

/// Create a semantic material surface using GPUI's cross-platform fallback
/// renderer.
///
/// This helper intentionally exposes semantic roles instead of platform APIs.
/// It returns a styled [`Div`] so callers can continue to compose layout and
/// children using normal GPUI patterns while the backend implementation evolves.
pub fn material_surface(window: &Window, style: MaterialStyle) -> Div {
    let appearance = DefaultAppearance::from(window.appearance());
    let active = window.is_window_active();
    let background = material_background(appearance, active, style);
    let border = material_border_color(appearance, active, style);
    let radius = material_corner_radius(style.role);
    let should_shadow = !matches!(
        style.role,
        MaterialRole::ScrollEdge | MaterialRole::BackgroundExtension
    );

    let mut surface = div()
        .rounded(px(radius))
        .border(px(1.0))
        .border_color(border)
        .bg(background);

    if should_shadow {
        surface = surface.shadow_sm();
    }

    surface
}

fn material_corner_radius(role: MaterialRole) -> f32 {
    match role {
        MaterialRole::Sidebar | MaterialRole::Toolbar | MaterialRole::Inspector => 18.0,
        MaterialRole::Overlay => 20.0,
        MaterialRole::SearchField => 14.0,
        MaterialRole::ScrollEdge => 16.0,
        MaterialRole::BackgroundExtension => 18.0,
        MaterialRole::GroupedControls => 16.0,
    }
}

fn material_background(
    appearance: DefaultAppearance,
    active: bool,
    style: MaterialStyle,
) -> Background {
    let (top, bottom) = match appearance {
        DefaultAppearance::Light => light_material_pair(active, style),
        DefaultAppearance::Dark => dark_material_pair(active, style),
    };
    linear_gradient(
        180.0,
        linear_color_stop(top, 0.0),
        linear_color_stop(bottom, 1.0),
    )
}

fn material_border_color(
    appearance: DefaultAppearance,
    active: bool,
    style: MaterialStyle,
) -> crate::Rgba {
    let alpha = match (style.emphasis, active, style.interactive) {
        (MaterialEmphasis::High, true, true) => 0.32,
        (MaterialEmphasis::High, true, false) => 0.24,
        (MaterialEmphasis::Medium, true, true) => 0.24,
        (MaterialEmphasis::Medium, true, false) => 0.18,
        (MaterialEmphasis::Low, true, _) => 0.12,
        (MaterialEmphasis::High, false, _) => 0.18,
        (MaterialEmphasis::Medium, false, _) => 0.14,
        (MaterialEmphasis::Low, false, _) => 0.1,
    };

    match appearance {
        DefaultAppearance::Light => opaque_grey(1.0, alpha).to_rgb(),
        DefaultAppearance::Dark => opaque_grey(0.92, alpha).to_rgb(),
    }
}

fn light_material_pair(active: bool, style: MaterialStyle) -> (crate::Hsla, crate::Hsla) {
    let base_alpha = alpha_for_style(active, style);
    let tint_shift = if style.tinted { 0.04 } else { 0.0 };

    match style.role {
        MaterialRole::Sidebar | MaterialRole::Inspector => (
            hsla(0.58, 0.10 + tint_shift, 0.98, base_alpha),
            hsla(0.58, 0.12 + tint_shift, 0.93, base_alpha + 0.05),
        ),
        MaterialRole::Toolbar | MaterialRole::GroupedControls => (
            hsla(0.58, 0.08 + tint_shift, 0.99, base_alpha),
            hsla(0.58, 0.10 + tint_shift, 0.95, base_alpha + 0.04),
        ),
        MaterialRole::Overlay => (
            hsla(0.58, 0.10 + tint_shift, 1.0, (base_alpha + 0.08).min(0.96)),
            hsla(0.58, 0.12 + tint_shift, 0.95, (base_alpha + 0.1).min(0.98)),
        ),
        MaterialRole::SearchField => (
            hsla(0.58, 0.05 + tint_shift, 0.99, (base_alpha - 0.06).max(0.24)),
            hsla(0.58, 0.07 + tint_shift, 0.95, (base_alpha - 0.02).max(0.28)),
        ),
        MaterialRole::ScrollEdge => (
            hsla(0.58, 0.04 + tint_shift, 0.98, (base_alpha - 0.12).max(0.16)),
            hsla(0.58, 0.05 + tint_shift, 0.92, (base_alpha + 0.02).max(0.22)),
        ),
        MaterialRole::BackgroundExtension => (
            hsla(0.58, 0.04 + tint_shift, 0.97, (base_alpha - 0.16).max(0.10)),
            hsla(0.58, 0.06 + tint_shift, 0.92, (base_alpha - 0.08).max(0.14)),
        ),
    }
}

fn dark_material_pair(active: bool, style: MaterialStyle) -> (crate::Hsla, crate::Hsla) {
    let base_alpha = alpha_for_style(active, style);
    let accent_saturation = if style.tinted { 0.18 } else { 0.08 };

    match style.role {
        MaterialRole::Sidebar | MaterialRole::Inspector => (
            hsla(0.60, accent_saturation, 0.18, base_alpha),
            hsla(0.60, accent_saturation + 0.02, 0.12, base_alpha + 0.07),
        ),
        MaterialRole::Toolbar | MaterialRole::GroupedControls => (
            hsla(0.60, accent_saturation, 0.20, base_alpha - 0.02),
            hsla(0.60, accent_saturation + 0.02, 0.14, base_alpha + 0.05),
        ),
        MaterialRole::Overlay => (
            hsla(0.60, accent_saturation + 0.02, 0.18, (base_alpha + 0.08).min(0.98)),
            hsla(0.60, accent_saturation + 0.04, 0.10, (base_alpha + 0.12).min(0.99)),
        ),
        MaterialRole::SearchField => (
            hsla(0.60, accent_saturation - 0.02, 0.22, (base_alpha - 0.10).max(0.20)),
            hsla(0.60, accent_saturation, 0.14, (base_alpha - 0.04).max(0.24)),
        ),
        MaterialRole::ScrollEdge => (
            hsla(0.60, accent_saturation - 0.03, 0.18, (base_alpha - 0.14).max(0.16)),
            hsla(0.60, accent_saturation, 0.10, (base_alpha - 0.02).max(0.22)),
        ),
        MaterialRole::BackgroundExtension => (
            hsla(0.60, accent_saturation - 0.04, 0.16, (base_alpha - 0.18).max(0.08)),
            hsla(0.60, accent_saturation - 0.02, 0.09, (base_alpha - 0.10).max(0.12)),
        ),
    }
}

fn alpha_for_style(active: bool, style: MaterialStyle) -> f32 {
    let emphasis: f32 = match style.emphasis {
        MaterialEmphasis::Low => 0.0,
        MaterialEmphasis::Medium => 0.06,
        MaterialEmphasis::High => 0.12,
    };
    let role: f32 = match style.role {
        MaterialRole::Sidebar | MaterialRole::Inspector => 0.66,
        MaterialRole::Toolbar | MaterialRole::GroupedControls => 0.62,
        MaterialRole::Overlay => 0.76,
        MaterialRole::SearchField => 0.54,
        MaterialRole::ScrollEdge => 0.42,
        MaterialRole::BackgroundExtension => 0.34,
    };
    let variant: f32 = match style.variant {
        MaterialVariant::Adaptive => 0.0,
        MaterialVariant::Clear => -0.14,
        MaterialVariant::OpaqueFallback => 0.18,
    };
    let interaction: f32 = if style.interactive { 0.04 } else { 0.0 };
    let activation: f32 = if active { 0.0 } else { -0.08 };

    (role + emphasis + variant + interaction + activation).clamp(0.08_f32, 0.96_f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_style_defaults_are_semantic_and_stable() {
        let style = MaterialStyle::sidebar();
        assert_eq!(style.role, MaterialRole::Sidebar);
        assert_eq!(style.variant, MaterialVariant::Adaptive);
        assert_eq!(style.emphasis, MaterialEmphasis::Medium);
        assert!(!style.tinted);
        assert!(!style.interactive);
    }

    #[test]
    fn platform_material_capabilities_start_conservative() {
        let capabilities = platform_material_capabilities();
        assert_eq!(capabilities, PlatformMaterialCapabilities::default());
    }
}
