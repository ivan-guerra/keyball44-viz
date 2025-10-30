use anyhow::Result;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct Layer {
    pub index: usize,
    pub keys: Vec<Vec<String>>,
}

pub fn parse_layers(content: &str) -> Result<Vec<Layer>> {
    let mut layers = Vec::new();
    let mut in_keymaps = false;
    let mut in_layer = false;
    let mut current_keys = Vec::new();
    let mut layer_count = 0;

    // Regex to match LAYOUT or LAYOUT_* variants followed by (
    let layout_regex = Regex::new(r"LAYOUT(_\w+)?\s*\(").unwrap();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.starts_with("//") || trimmed.is_empty() {
            continue;
        }

        // Detect start of keymaps array
        if trimmed.contains("const uint16_t PROGMEM keymaps") {
            in_keymaps = true;
        }

        if !in_keymaps {
            continue;
        }

        // Detect end of keymaps array (only the closing one, not nested ones)
        if trimmed.starts_with("};") && !in_layer {
            break;
        }

        // Detect start of a layer - match LAYOUT or LAYOUT_* variants
        if layout_regex.is_match(trimmed) {
            in_layer = true;
            continue;
        }

        // Detect end of a layer
        if in_layer && (trimmed.starts_with("),") || trimmed == ")," || trimmed == ")") {
            in_layer = false;

            // Add the layer
            if !current_keys.is_empty() {
                layers.push(Layer {
                    index: layer_count,
                    keys: current_keys.clone(),
                });
                current_keys.clear();
                layer_count += 1;
            }
            continue;
        }

        // Parse key data when inside a layer
        if in_layer {
            let keys = parse_keys_with_parens(trimmed);

            if !keys.is_empty() {
                current_keys.push(keys);
            }
        }
    }

    Ok(layers)
}

pub fn is_empty_key(key: &str) -> bool {
    key.chars().all(|c| c == '_')
}

fn parse_keys_with_parens(line: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut current_key = String::new();
    let mut paren_depth = 0;

    let trimmed = line.trim_end_matches(',');

    for ch in trimmed.chars() {
        match ch {
            '(' => {
                paren_depth += 1;
                current_key.push(ch);
            }
            ')' => {
                paren_depth -= 1;
                current_key.push(ch);
            }
            ',' => {
                if paren_depth == 0 {
                    // Only split on commas outside of parentheses
                    let key = current_key.trim().to_string();
                    if !key.is_empty() {
                        keys.push(key);
                    }
                    current_key.clear();
                } else {
                    current_key.push(ch);
                }
            }
            _ => {
                current_key.push(ch);
            }
        }
    }

    // Don't forget the last key
    let key = current_key.trim().to_string();
    if !key.is_empty() {
        keys.push(key);
    }

    keys
}

pub fn generate_svg(layers: &[Layer]) -> String {
    const KEY_HEIGHT: f32 = 60.0;
    const KEY_SPACING: f32 = 5.0;
    const SPLIT_GAP: f32 = 40.0;
    const LAYER_SPACING: f32 = 120.0;
    const MARGIN: f32 = 20.0;
    const FONT_SIZE: f32 = 11.0;
    const CHAR_WIDTH: f32 = 7.0; // Approximate width per character in monospace font
    const KEY_PADDING: f32 = 10.0; // Padding inside the key

    // Calculate the width needed for the longest key label
    let max_label_len = layers
        .iter()
        .flat_map(|l| l.keys.iter())
        .flat_map(|row| row.iter())
        .map(|key| key.len())
        .max()
        .unwrap_or(8);

    let key_width = (max_label_len as f32 * CHAR_WIDTH + KEY_PADDING * 2.0).max(60.0);

    // Keyball44 layout structure
    // Each row: (left_keys, left_offset, right_keys, right_offset)
    // Left half bottom row staggered right by 2, right half bottom row staggered left by 1
    let layout: [(i32, f32, i32, f32); 4] = [
        (6, 0.0, 6, 0.0),  // Row 0: 6 left + 6 right
        (6, 0.0, 6, 0.0),  // Row 1: 6 left + 6 right
        (6, 0.0, 6, 0.0),  // Row 2: 6 left + 6 right
        (5, 2.0, 3, -1.0), // Row 3: 5 left (offset +2) + 3 right (offset -1)
    ];

    let left_width = 8.0 * (key_width + KEY_SPACING); // Account for stagger
    let right_width = 6.0 * (key_width + KEY_SPACING);
    let svg_width = MARGIN * 2.0 + left_width + SPLIT_GAP + right_width;

    let mut svg = String::new();
    let mut total_height = MARGIN;

    // Start SVG (we'll update height later)
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
        svg_width as i32, 0, svg_width as i32, 0
    ));
    svg.push('\n');

    // Add styles
    svg.push_str(&format!(
        r#"<style>
        .key {{ fill: white; stroke: #333; stroke-width: 2; }}
        .key-text {{ fill: #333; font-family: monospace; font-size: {}px; text-anchor: middle; }}
        .layer-title {{ fill: #333; font-family: sans-serif; font-size: 20px; font-weight: bold; }}
    </style>"#,
        FONT_SIZE
    ));
    svg.push('\n');

    // Render each layer
    for layer in layers {
        let y_offset = total_height;

        // Layer title
        svg.push_str(&format!(
            r#"<text x="{}" y="{}" class="layer-title">Layer {}</text>"#,
            MARGIN,
            y_offset + 30.0,
            layer.index
        ));
        svg.push('\n');

        let layer_start_y = y_offset + 50.0;

        // Flatten all keys from the layer
        let all_keys: Vec<&String> = layer.keys.iter().flat_map(|row| row.iter()).collect();

        let mut key_index = 0;
        let right_start_x = MARGIN + left_width + SPLIT_GAP;

        // Render each row (left + right together)
        for (row_idx, &(left_keys, left_offset, right_keys, right_offset)) in
            layout.iter().enumerate()
        {
            let y = layer_start_y + row_idx as f32 * (KEY_HEIGHT + KEY_SPACING);

            // Render left side of this row
            let left_x_offset = left_offset * (key_width + KEY_SPACING);
            for col_idx in 0..left_keys {
                if key_index >= all_keys.len() {
                    break;
                }

                let x = MARGIN + left_x_offset + col_idx as f32 * (key_width + KEY_SPACING);
                let key = all_keys[key_index];

                // Draw key
                svg.push_str(&format!(
                    r#"<rect class="key" x="{}" y="{}" width="{}" height="{}" rx="5"/>"#,
                    x, y, key_width, KEY_HEIGHT
                ));
                svg.push('\n');

                // Draw key label
                svg.push_str(&format!(
                    r#"<text class="key-text" x="{}" y="{}">{}</text>"#,
                    x + key_width / 2.0,
                    y + KEY_HEIGHT / 2.0 + FONT_SIZE / 3.0,
                    escape_xml(key)
                ));
                svg.push('\n');

                key_index += 1;
            }

            // Render right side of this row
            let right_x_offset = right_offset * (key_width + KEY_SPACING);

            // Special handling for bottom row (row 3) - last key aligned with 4th key above
            if row_idx == 3 {
                for col_idx in 0..right_keys {
                    if key_index >= all_keys.len() {
                        break;
                    }

                    let x = if col_idx < 2 {
                        // First 2 keys: staggered left by 1
                        right_start_x + right_x_offset + col_idx as f32 * (key_width + KEY_SPACING)
                    } else {
                        // Last key: aligned with 4th key in row above
                        right_start_x + 3.0 * (key_width + KEY_SPACING)
                    };

                    let key = all_keys[key_index];

                    // Draw key
                    svg.push_str(&format!(
                        r#"<rect class="key" x="{}" y="{}" width="{}" height="{}" rx="5"/>"#,
                        x, y, key_width, KEY_HEIGHT
                    ));
                    svg.push('\n');

                    // Draw key label
                    svg.push_str(&format!(
                        r#"<text class="key-text" x="{}" y="{}">{}</text>"#,
                        x + key_width / 2.0,
                        y + KEY_HEIGHT / 2.0 + FONT_SIZE / 3.0,
                        escape_xml(key)
                    ));
                    svg.push('\n');

                    key_index += 1;
                }
            } else {
                for col_idx in 0..right_keys {
                    if key_index >= all_keys.len() {
                        break;
                    }

                    let x =
                        right_start_x + right_x_offset + col_idx as f32 * (key_width + KEY_SPACING);
                    let key = all_keys[key_index];

                    // Draw key
                    svg.push_str(&format!(
                        r#"<rect class="key" x="{}" y="{}" width="{}" height="{}" rx="5"/>"#,
                        x, y, key_width, KEY_HEIGHT
                    ));
                    svg.push('\n');

                    // Draw key label
                    svg.push_str(&format!(
                        r#"<text class="key-text" x="{}" y="{}">{}</text>"#,
                        x + key_width / 2.0,
                        y + KEY_HEIGHT / 2.0 + FONT_SIZE / 3.0,
                        escape_xml(key)
                    ));
                    svg.push('\n');

                    key_index += 1;
                }
            }
        }

        let layer_height = 4.0 * (KEY_HEIGHT + KEY_SPACING) + 50.0;
        total_height += layer_height + LAYER_SPACING;
    }

    svg.push_str("</svg>");

    // Update the height in the SVG header
    let total_height_int = total_height as i32;
    svg = svg.replace(
        &format!(r#"height="{}" viewBox="0 0 {} {}"#, 0, svg_width as i32, 0),
        &format!(
            r#"height="{}" viewBox="0 0 {} {}"#,
            total_height_int, svg_width as i32, total_height_int
        ),
    );

    svg
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
