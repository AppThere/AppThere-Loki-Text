use std::collections::HashMap;

use common_core::StyleDefinition;

use crate::FontAsset;

/// Generate the EPUB stylesheet from named styles and embedded fonts.
pub(crate) fn generate_css(
    styles: &HashMap<String, StyleDefinition>,
    fonts: &[FontAsset],
) -> String {
    let mut css = String::new();

    // @font-face rules for embedded fonts
    for font in fonts {
        css.push_str(&format!(
            "@font-face {{\n  font-family: '{}';\n  src: url('../Fonts/{}');\n}}\n\n",
            font.family_name, font.filename
        ));
    }

    // One CSS class per named style
    for (name, style) in styles {
        let class_name = name.replace(' ', "-");
        css.push_str(&format!(".style-{} {{\n", class_name));

        for (key, value) in &style.attributes {
            let css_prop = odf_to_css_property(key);
            if !css_prop.is_empty() {
                css.push_str(&format!("  {}: {};\n", css_prop, value));
            }
        }

        if let Some(transform) = &style.text_transform {
            css.push_str(&format!("  text-transform: {};\n", transform));
        }

        css.push_str("}\n\n");
    }

    css
}

/// Map an ODF property name (prefixed) to its CSS equivalent.
/// Returns an empty string for properties with no direct CSS mapping.
fn odf_to_css_property(odf_prop: &str) -> &'static str {
    match odf_prop {
        // Typography
        "fo:font-family" => "font-family",
        "style:font-name" => "font-family",  // G10: alternate ODF font property
        "fo:font-size" => "font-size",
        "fo:font-weight" => "font-weight",
        "fo:font-style" => "font-style",
        "fo:font-variant" => "font-variant",  // G10
        "fo:letter-spacing" => "letter-spacing", // G10
        // Decoration
        "fo:text-decoration" => "text-decoration", // G10
        "fo:text-transform" => "text-transform",
        // Colour (G10)
        "fo:color" => "color",
        "fo:background-color" => "background-color",
        // Text layout
        "fo:text-align" => "text-align",
        "fo:text-indent" => "text-indent",
        "fo:line-height" => "line-height",
        // Margins / spacing
        "fo:margin-top" => "margin-top",
        "fo:margin-bottom" => "margin-bottom",
        "fo:margin-left" => "margin-left",
        "fo:margin-right" => "margin-right",
        "fo:padding" => "padding",
        "fo:padding-top" => "padding-top",
        "fo:padding-bottom" => "padding-bottom",
        "fo:padding-left" => "padding-left",
        "fo:padding-right" => "padding-right",
        // Borders
        "fo:border" => "border",
        "fo:border-top" => "border-top",
        "fo:border-bottom" => "border-bottom",
        "fo:border-left" => "border-left",
        "fo:border-right" => "border-right",
        _ => "",
    }
}
