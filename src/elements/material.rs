use crate::{
    AnyElement, App, Background, Bounds, Div, Element, ElementId, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, ParentElement, StyleRefinement, Window,
    colors::DefaultAppearance, div, hsla, linear_color_stop, linear_gradient, opaque_grey, px,
    styled::Styled,
};
#[cfg(target_os = "macos")]
use objc::runtime::Class;
use refineable::Refineable as _;
use smallvec::SmallVec;
use std::mem;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct MaterialFallbackPreferences {
    reduced_transparency: bool,
    platform: MaterialFallbackPlatform,
}

impl Default for MaterialFallbackPreferences {
    fn default() -> Self {
        Self {
            reduced_transparency: false,
            platform: MaterialFallbackPlatform::current(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MaterialFallbackPlatform {
    MacOS,
    Windows,
    Linux,
    Other,
}

impl MaterialFallbackPlatform {
    const fn current() -> Self {
        #[cfg(target_os = "macos")]
        {
            Self::MacOS
        }

        #[cfg(windows)]
        {
            Self::Windows
        }

        #[cfg(target_os = "linux")]
        {
            Self::Linux
        }

        #[cfg(not(any(target_os = "macos", windows, target_os = "linux")))]
        {
            Self::Other
        }
    }
}

/// Return the native material capabilities currently implemented by GPUI.
pub fn platform_material_capabilities() -> PlatformMaterialCapabilities {
    let native_glass = runtime_native_glass_supported();
    PlatformMaterialCapabilities {
        native_glass,
        background_extension: native_glass,
        scroll_edge_effects: native_glass,
        adaptive_grouped_controls: true,
        ..PlatformMaterialCapabilities::default()
    }
}

/// Create a semantic material surface using GPUI's semantic material backend.
///
/// This helper intentionally exposes semantic roles instead of platform APIs.
/// It returns a styled parent element so callers can continue to compose
/// layout and children using normal GPUI patterns while the backend
/// implementation evolves.
#[track_caller]
pub fn material_surface(_window: &Window, style: MaterialStyle) -> MaterialSurface {
    let source = core::panic::Location::caller();
    MaterialSurface {
        material_style: style,
        style: StyleRefinement::default(),
        children: SmallVec::default(),
        source,
        element_id: ElementId::CodeLocation(*source),
    }
}

/// A semantic material surface element.
pub struct MaterialSurface {
    material_style: MaterialStyle,
    style: StyleRefinement,
    children: SmallVec<[AnyElement; 2]>,
    source: &'static core::panic::Location<'static>,
    element_id: ElementId,
}

impl MaterialSurface {
    fn build_surface(&mut self, window: &Window) -> Div {
        let preferences = MaterialFallbackPreferences::default();
        let appearance = DefaultAppearance::from(window.appearance());
        let active = window.is_window_active();
        let resolved_style = resolve_material_style(self.material_style, active, preferences);
        let background = material_background(appearance, active, resolved_style, preferences);
        let border = material_border_color(appearance, active, resolved_style, preferences);
        let radius = material_corner_radius(resolved_style.role);
        let should_shadow = !matches!(
            resolved_style.role,
            MaterialRole::ScrollEdge | MaterialRole::BackgroundExtension
        );
        let use_native_backend = should_use_native_material_surface(window, resolved_style);

        let mut surface = div()
            .rounded(px(radius))
            .border(px(1.0))
            .border_color(border);

        if !use_native_backend {
            surface = surface.bg(background);
        }

        if should_shadow {
            surface = surface.shadow_sm();
        }

        surface.style().refine(&self.style);
        surface.children(mem::take(&mut self.children))
    }
}

impl ParentElement for MaterialSurface {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for MaterialSurface {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl IntoElement for MaterialSurface {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for MaterialSurface {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.element_id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        Some(self.source)
    }

    fn request_layout(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut surface = self.build_surface(window).into_any_element();
        let layout_id = surface.request_layout(window, cx);
        (layout_id, surface)
    }

    fn prepaint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<crate::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let resolved_style = resolve_material_style(
            self.material_style,
            window.is_window_active(),
            MaterialFallbackPreferences::default(),
        );

        if should_use_native_material_surface(window, resolved_style)
            && let Some(global_id) = global_id
        {
            window.sync_native_material_surface(global_id, bounds, resolved_style);
        }

        request_layout.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<crate::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        request_layout.paint(window, cx);
    }
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
    preferences: MaterialFallbackPreferences,
) -> Background {
    let (top, bottom) = match appearance {
        DefaultAppearance::Light => light_material_pair(active, style, preferences),
        DefaultAppearance::Dark => dark_material_pair(active, style, preferences),
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
    preferences: MaterialFallbackPreferences,
) -> crate::Rgba {
    let mut alpha = match (style.emphasis, active, style.interactive) {
        (MaterialEmphasis::High, true, true) => 0.32,
        (MaterialEmphasis::High, true, false) => 0.24,
        (MaterialEmphasis::Medium, true, true) => 0.24,
        (MaterialEmphasis::Medium, true, false) => 0.18,
        (MaterialEmphasis::Low, true, _) => 0.12,
        (MaterialEmphasis::High, false, _) => 0.18,
        (MaterialEmphasis::Medium, false, _) => 0.14,
        (MaterialEmphasis::Low, false, _) => 0.1,
    };
    alpha = (alpha + platform_border_bias(preferences.platform, style.variant)).clamp(0.08, 0.4);

    match appearance {
        DefaultAppearance::Light => opaque_grey(1.0, alpha).to_rgb(),
        DefaultAppearance::Dark => opaque_grey(0.92, alpha).to_rgb(),
    }
}

fn light_material_pair(
    active: bool,
    style: MaterialStyle,
    preferences: MaterialFallbackPreferences,
) -> (crate::Hsla, crate::Hsla) {
    let base_alpha = alpha_for_style(active, style, preferences);
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

fn dark_material_pair(
    active: bool,
    style: MaterialStyle,
    preferences: MaterialFallbackPreferences,
) -> (crate::Hsla, crate::Hsla) {
    let base_alpha = alpha_for_style(active, style, preferences);
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
            hsla(
                0.60,
                accent_saturation + 0.02,
                0.18,
                (base_alpha + 0.08).min(0.98),
            ),
            hsla(
                0.60,
                accent_saturation + 0.04,
                0.10,
                (base_alpha + 0.12).min(0.99),
            ),
        ),
        MaterialRole::SearchField => (
            hsla(
                0.60,
                accent_saturation - 0.02,
                0.22,
                (base_alpha - 0.10).max(0.20),
            ),
            hsla(0.60, accent_saturation, 0.14, (base_alpha - 0.04).max(0.24)),
        ),
        MaterialRole::ScrollEdge => (
            hsla(
                0.60,
                accent_saturation - 0.03,
                0.18,
                (base_alpha - 0.14).max(0.16),
            ),
            hsla(0.60, accent_saturation, 0.10, (base_alpha - 0.02).max(0.22)),
        ),
        MaterialRole::BackgroundExtension => (
            hsla(
                0.60,
                accent_saturation - 0.04,
                0.16,
                (base_alpha - 0.18).max(0.08),
            ),
            hsla(
                0.60,
                accent_saturation - 0.02,
                0.09,
                (base_alpha - 0.10).max(0.12),
            ),
        ),
    }
}

fn alpha_for_style(
    active: bool,
    style: MaterialStyle,
    preferences: MaterialFallbackPreferences,
) -> f32 {
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
    let platform = platform_alpha_bias(preferences.platform, style.variant);

    (role + emphasis + variant + interaction + activation + platform).clamp(0.08_f32, 0.96_f32)
}

fn resolve_material_style(
    mut style: MaterialStyle,
    active: bool,
    preferences: MaterialFallbackPreferences,
) -> MaterialStyle {
    if preferences.reduced_transparency {
        style.variant = MaterialVariant::OpaqueFallback;
    } else if !active && matches!(style.variant, MaterialVariant::Clear) {
        style.variant = MaterialVariant::Adaptive;
    }

    if !active && style.interactive {
        style.emphasis = soften_emphasis(style.emphasis);
    }

    style
}

fn soften_emphasis(emphasis: MaterialEmphasis) -> MaterialEmphasis {
    match emphasis {
        MaterialEmphasis::High => MaterialEmphasis::Medium,
        MaterialEmphasis::Medium | MaterialEmphasis::Low => MaterialEmphasis::Low,
    }
}

fn platform_alpha_bias(platform: MaterialFallbackPlatform, variant: MaterialVariant) -> f32 {
    match (platform, variant) {
        (MaterialFallbackPlatform::MacOS, _) => 0.0,
        (MaterialFallbackPlatform::Windows, MaterialVariant::Clear) => 0.06,
        (MaterialFallbackPlatform::Windows, _) => 0.03,
        (MaterialFallbackPlatform::Linux, MaterialVariant::Clear) => 0.12,
        (MaterialFallbackPlatform::Linux, _) => 0.08,
        (MaterialFallbackPlatform::Other, MaterialVariant::Clear) => 0.10,
        (MaterialFallbackPlatform::Other, _) => 0.06,
    }
}

fn platform_border_bias(platform: MaterialFallbackPlatform, variant: MaterialVariant) -> f32 {
    match (platform, variant) {
        (MaterialFallbackPlatform::MacOS, _) => 0.0,
        (MaterialFallbackPlatform::Windows, MaterialVariant::Clear) => 0.04,
        (MaterialFallbackPlatform::Windows, _) => 0.02,
        (MaterialFallbackPlatform::Linux, MaterialVariant::Clear) => 0.06,
        (MaterialFallbackPlatform::Linux, _) => 0.03,
        (MaterialFallbackPlatform::Other, MaterialVariant::Clear) => 0.05,
        (MaterialFallbackPlatform::Other, _) => 0.03,
    }
}

fn should_use_native_material_surface(window: &Window, style: MaterialStyle) -> bool {
    window.supports_native_material_surface(style)
}

#[cfg(target_os = "macos")]
fn runtime_native_glass_supported() -> bool {
    Class::get("NSGlassEffectView").is_some()
}

#[cfg(not(target_os = "macos"))]
fn runtime_native_glass_supported() -> bool {
    false
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
    fn platform_material_capabilities_report_current_support() {
        let capabilities = platform_material_capabilities();
        assert_eq!(capabilities.native_glass, runtime_native_glass_supported());
        assert_eq!(
            capabilities.background_extension,
            runtime_native_glass_supported()
        );
        assert_eq!(
            capabilities.scroll_edge_effects,
            runtime_native_glass_supported()
        );
        assert!(!capabilities.concentric_layout_regions);
        assert!(capabilities.adaptive_grouped_controls);
    }

    #[test]
    fn reduced_transparency_forces_opaque_variant() {
        let style = resolve_material_style(
            MaterialStyle::overlay().variant(MaterialVariant::Clear),
            true,
            MaterialFallbackPreferences {
                reduced_transparency: true,
                platform: MaterialFallbackPlatform::MacOS,
            },
        );

        assert_eq!(style.variant, MaterialVariant::OpaqueFallback);
    }

    #[test]
    fn inactive_clear_materials_fall_back_to_adaptive_variant() {
        let style = resolve_material_style(
            MaterialStyle::toolbar().variant(MaterialVariant::Clear),
            false,
            MaterialFallbackPreferences {
                reduced_transparency: false,
                platform: MaterialFallbackPlatform::MacOS,
            },
        );

        assert_eq!(style.variant, MaterialVariant::Adaptive);
    }

    #[test]
    fn inactive_interactive_materials_soften_emphasis() {
        let style = resolve_material_style(
            MaterialStyle::grouped_controls()
                .interactive(true)
                .emphasis(MaterialEmphasis::High),
            false,
            MaterialFallbackPreferences {
                reduced_transparency: false,
                platform: MaterialFallbackPlatform::MacOS,
            },
        );

        assert_eq!(style.emphasis, MaterialEmphasis::Medium);
    }

    #[test]
    fn linux_fallback_bias_is_more_opaque_than_macos() {
        let style = MaterialStyle::toolbar().variant(MaterialVariant::Clear);
        let mac_alpha = alpha_for_style(
            true,
            style,
            MaterialFallbackPreferences {
                reduced_transparency: false,
                platform: MaterialFallbackPlatform::MacOS,
            },
        );
        let linux_alpha = alpha_for_style(
            true,
            style,
            MaterialFallbackPreferences {
                reduced_transparency: false,
                platform: MaterialFallbackPlatform::Linux,
            },
        );

        assert!(linux_alpha > mac_alpha);
    }

    #[test]
    fn non_macos_platforms_bias_clear_materials_toward_more_opacity() {
        let style = MaterialStyle::overlay().variant(MaterialVariant::Clear);
        let mac_alpha = alpha_for_style(
            true,
            style,
            MaterialFallbackPreferences {
                reduced_transparency: false,
                platform: MaterialFallbackPlatform::MacOS,
            },
        );
        let windows_alpha = alpha_for_style(
            true,
            style,
            MaterialFallbackPreferences {
                reduced_transparency: false,
                platform: MaterialFallbackPlatform::Windows,
            },
        );
        let other_alpha = alpha_for_style(
            true,
            style,
            MaterialFallbackPreferences {
                reduced_transparency: false,
                platform: MaterialFallbackPlatform::Other,
            },
        );

        assert!(windows_alpha > mac_alpha);
        assert!(other_alpha > mac_alpha);
    }
}
