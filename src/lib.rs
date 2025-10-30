//! A CLI tool that parses QMK keymap.c files for the Keyball44 keyboard and
//! generates an SVG visualization of all layers with color-coded keys.
use anyhow::Result;
use regex::Regex;
use svg::{
    node::element::{Definitions, LinearGradient, Rectangle, Stop, Style, Text},
    Document,
};

/// Represents a single keymap layer in the keyboard layout.
///
/// Each layer contains an index identifier and a 2D grid of key labels,
/// where each inner vector represents a row of keys on the keyboard.
#[derive(Debug, Clone)]
pub struct Layer {
    /// The layer number/identifier (e.g., 0 for base layer, 1 for first modifier layer)
    pub index: usize,
    /// A 2D vector representing rows and columns of key labels on this layer
    pub keys: Vec<Vec<String>>,
}

/// Parses QMK keymap C code to extract layer definitions.
///
/// This function reads through QMK firmware keymap source code and extracts
/// all keyboard layers defined within the `keymaps` array. It handles various
/// LAYOUT macro formats (LAYOUT, LAYOUT_split_3x5_3, etc.) and parses the
/// key definitions within each layer.
///
/// # Arguments
///
/// * `content` - A string slice containing the QMK keymap C source code
///
/// # Returns
///
/// * `Result<Vec<Layer>>` - A vector of parsed Layer structs, or an error if parsing fails
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

/// Checks if a key string represents an empty key.
///
/// A key is considered empty if it consists entirely of underscore characters.
/// This is commonly used in keyboard layouts to represent unassigned or blank keys.
///
/// # Arguments
///
/// * `key` - A string slice representing the key to check
///
/// # Returns
///
/// Returns `true` if the key contains only underscores, `false` otherwise.
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

fn add_gradients(document: Document) -> Document {
    let mut defs = Definitions::new();

    // Default key gradient (GMK WoB/BoW style - light grey)
    let key_gradient = LinearGradient::new()
        .set("id", "keyGradient")
        .set("x1", "0%")
        .set("y1", "0%")
        .set("x2", "0%")
        .set("y2", "100%")
        .add(Stop::new().set("offset", "0%").set("stop-color", "#e8e8e8"))
        .add(
            Stop::new()
                .set("offset", "100%")
                .set("stop-color", "#d0d0d0"),
        );

    // Layer-specific gradients using GMK-inspired colors
    let layer_colors = vec![
        ("#7cb0d9", "#5a8fb8"), // Layer 1 - GMK Blue (Dolch/Nautilus blue)
        ("#b888c4", "#9668a8"), // Layer 2 - GMK Purple (Laser purple)
        ("#d97c7c", "#c25858"), // Layer 3 - GMK Red (Red Samurai red)
        ("#e8a87c", "#d18a58"), // Layer 4 - GMK Orange (Camping orange)
        ("#7ec4a8", "#5ca888"), // Layer 5 - GMK Teal (Cyan/Miami teal)
        ("#88c47c", "#68a858"), // Layer 6 - GMK Green (Botanical green)
        ("#d4c47c", "#b8a858"), // Layer 7 - GMK Yellow (Honey yellow)
        ("#a8a8a8", "#888888"), // Layer 8 - GMK Dark Grey (modifier grey)
    ];

    for (idx, (light, dark)) in layer_colors.iter().enumerate() {
        let gradient = LinearGradient::new()
            .set("id", format!("layer{}Gradient", idx + 1))
            .set("x1", "0%")
            .set("y1", "0%")
            .set("x2", "0%")
            .set("y2", "100%")
            .add(Stop::new().set("offset", "0%").set("stop-color", *light))
            .add(Stop::new().set("offset", "100%").set("stop-color", *dark));
        defs = defs.add(gradient);
    }

    // Special key gradient (GMK accent teal)
    let special_gradient = LinearGradient::new()
        .set("id", "specialGradient")
        .set("x1", "0%")
        .set("y1", "0%")
        .set("x2", "0%")
        .set("y2", "100%")
        .add(Stop::new().set("offset", "0%").set("stop-color", "#7ec4a8"))
        .add(
            Stop::new()
                .set("offset", "100%")
                .set("stop-color", "#5ca888"),
        );

    defs = defs.add(key_gradient);
    defs = defs.add(special_gradient);

    document.add(defs)
}

fn get_key_class(key: &str, layer_index: usize) -> String {
    if is_empty_key(key) {
        return "key key-empty".to_string();
    }

    // For Layer 0, check if it's a layer switch modifier
    if layer_index == 0 {
        if let Some(layer_num) = extract_layer_number(key) {
            return format!("key key-layer{}", layer_num);
        }

        // Check for special functions
        if key.starts_with("RGB_")
            || key.starts_with("BL_")
            || key.starts_with("RESET")
            || key.starts_with("QK_")
        {
            return "key key-special".to_string();
        }

        // Default for Layer 0 non-modifier keys
        return "key".to_string();
    }

    // For other layers, all non-empty keys get the layer color
    format!("key key-layer{}", layer_index)
}

fn extract_layer_number(key: &str) -> Option<usize> {
    // Extract layer number from layer switching functions
    if key.starts_with("MO(")
        || key.starts_with("TO(")
        || key.starts_with("TG(")
        || key.starts_with("TT(")
        || key.starts_with("OSL(")
        || key.starts_with("DF(")
    {
        let start = key.find('(')? + 1;
        let end = key.find(')')?;
        key[start..end].parse().ok()
    } else if key.starts_with("LT(") || key.starts_with("LM(") {
        let start = key.find('(')? + 1;
        let comma = key.find(',')?;
        key[start..comma].trim().parse().ok()
    } else {
        None
    }
}

/// Generates an SVG visualization of keyboard layers.
///
/// Creates a comprehensive SVG document displaying multiple keyboard layers with
/// proper spacing, gradients, and interactive styling. Each layer is rendered
/// separately with its keys arranged according to the Keyball44 layout specification.
///
/// # Arguments
///
/// * `layers` - A slice of `Layer` structs containing the keyboard layout data
///
/// # Returns
///
/// A `String` containing the complete SVG document
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

    let mut total_height = MARGIN;

    // Calculate total height
    for _ in layers {
        let layer_height = 4.0 * (KEY_HEIGHT + KEY_SPACING) + 50.0;
        total_height += layer_height + LAYER_SPACING;
    }

    // Create SVG document
    let mut document = Document::new()
        .set("width", svg_width as i32)
        .set("height", total_height as i32)
        .set("viewBox", (0, 0, svg_width as i32, total_height as i32));

    // Add background
    let background = Rectangle::new()
        .set("width", "100%")
        .set("height", "100%")
        .set("fill", "#faf8f3");
    document = document.add(background);

    // Add enhanced styles with gradients, shadows, and color coding
    let style = Style::new(
        r#"
        .key {
            fill: url(#keyGradient);
            stroke: #2c3e50;
            stroke-width: 2;
            filter: drop-shadow(2px 2px 3px rgba(0,0,0,0.2));
            transition: all 0.3s ease;
        }
        .key:hover {
            filter: drop-shadow(3px 3px 5px rgba(0,0,0,0.3));
            transform: translateY(-2px);
        }
        .key-layer1 { fill: url(#layer1Gradient); }
        .key-layer2 { fill: url(#layer2Gradient); }
        .key-layer3 { fill: url(#layer3Gradient); }
        .key-layer4 { fill: url(#layer4Gradient); }
        .key-layer5 { fill: url(#layer5Gradient); }
        .key-layer6 { fill: url(#layer6Gradient); }
        .key-layer7 { fill: url(#layer7Gradient); }
        .key-layer8 { fill: url(#layer8Gradient); }
        .key-special { fill: url(#specialGradient); }
        .key-empty { fill: #ecf0f1; opacity: 0.5; }
        
        .key-text {
            fill: #2c3e50;
            font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', monospace;
            font-size: 11px;
            font-weight: 500;
            text-anchor: middle;
            pointer-events: none;
        }
        .layer-title {
            fill: #34495e;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            font-size: 20px;
            font-weight: 600;
            letter-spacing: -0.5px;
        }
    "#,
    );
    document = document.add(style);

    // Add gradient definitions
    document = add_gradients(document);

    let mut y_offset = MARGIN;

    for layer in layers {
        // Draw layer title
        let title = Text::new("")
            .set("class", "layer-title")
            .set("x", MARGIN)
            .set("y", y_offset)
            .add(svg::node::Text::new(format!("Layer {}", layer.index)));
        document = document.add(title);
        y_offset += 40.0;

        // Draw keys for each row
        for (row_idx, row) in layer.keys.iter().enumerate() {
            if row_idx >= layout.len() {
                continue;
            }

            let (left_count, left_offset, right_count, right_offset) = layout[row_idx];
            let y = y_offset + row_idx as f32 * (KEY_HEIGHT + KEY_SPACING);

            // Draw left half keys
            for (col_idx, key) in row.iter().take(left_count as usize).enumerate() {
                let x = MARGIN
                    + left_offset * (key_width + KEY_SPACING)
                    + col_idx as f32 * (key_width + KEY_SPACING);

                // Draw key
                let rect = Rectangle::new()
                    .set("class", get_key_class(key, layer.index))
                    .set("x", x)
                    .set("y", y)
                    .set("width", key_width)
                    .set("height", KEY_HEIGHT)
                    .set("rx", 5);
                document = document.add(rect);

                // Draw key label
                let text = Text::new("")
                    .set("class", "key-text")
                    .set("x", x + key_width / 2.0)
                    .set("y", y + KEY_HEIGHT / 2.0 + FONT_SIZE / 3.0)
                    .add(svg::node::Text::new(key.as_str()));
                document = document.add(text);
            }

            // Draw right half keys
            let right_start_idx = left_count as usize;
            let right_base_x = MARGIN + left_width + SPLIT_GAP;

            for (col_idx, key) in row
                .iter()
                .skip(right_start_idx)
                .take(right_count as usize)
                .enumerate()
            {
                let x = right_base_x
                    + right_offset * (key_width + KEY_SPACING)
                    + col_idx as f32 * (key_width + KEY_SPACING);

                if !is_empty_key(key) || row_idx < 3 {
                    // Draw key
                    let rect = Rectangle::new()
                        .set("class", get_key_class(key, layer.index))
                        .set("x", x)
                        .set("y", y)
                        .set("width", key_width)
                        .set("height", KEY_HEIGHT)
                        .set("rx", 5);
                    document = document.add(rect);

                    // Draw key label
                    let text = Text::new("")
                        .set("class", "key-text")
                        .set("x", x + key_width / 2.0)
                        .set("y", y + KEY_HEIGHT / 2.0 + FONT_SIZE / 3.0)
                        .add(svg::node::Text::new(key.as_str()));
                    document = document.add(text);
                }
            }

            // Draw remaining keys from row (if any beyond the split)
            let remaining_start = right_start_idx + right_count as usize;
            if remaining_start < row.len() {
                for (offset, key) in row.iter().skip(remaining_start).enumerate() {
                    let x = right_base_x
                        + (right_count as f32 + offset as f32) * (key_width + KEY_SPACING);

                    // Draw key
                    let rect = Rectangle::new()
                        .set("class", get_key_class(key, layer.index))
                        .set("x", x)
                        .set("y", y)
                        .set("width", key_width)
                        .set("height", KEY_HEIGHT)
                        .set("rx", 5);
                    document = document.add(rect);

                    // Draw key label
                    let text = Text::new("")
                        .set("class", "key-text")
                        .set("x", x + key_width / 2.0)
                        .set("y", y + KEY_HEIGHT / 2.0 + FONT_SIZE / 3.0)
                        .add(svg::node::Text::new(key.as_str()));
                    document = document.add(text);
                }
            }
        }

        y_offset += 4.0 * (KEY_HEIGHT + KEY_SPACING) + LAYER_SPACING;
    }

    document.to_string()
}
