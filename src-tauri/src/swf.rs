use kurbo::Point;
use serde::Deserialize;
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use svgtypes::{Color, PathParser, PathSegment, Transform};
use swf_emitter::emit_swf;
use swf_parser::parse_swf;
use swf_types::{
    fill_styles, shape_records, CapStyle, FillStyle, JoinStyle, LineStyle, Movie, Rect, SRgb8,
    Shape, ShapeRecord, ShapeStyles, StraightSRgba8, Tag,
};
use tauri::{command, AppHandle};
use xmlparser::{Token, Tokenizer};
use crate::ba2::{Ba2Path, extract_file_from_ba2, is_ba2_path};

#[derive(Debug, Deserialize)]
pub struct ModificationConfig {
    pub file: Option<Vec<ShapeSource>>,
    pub transparent: Option<Vec<u16>>,  // Shape IDs to make transparent
    pub swf: SwfModification,
}

#[derive(Debug, Deserialize)]
pub struct BatchProcessConfig {
    pub config_file: String,           // Path to the main configuration file
    pub output_directory: String,      // Directory to save processed files
    pub ba2_path: Option<String>,      // User-selected BA2 file path (if using BA2)
}

#[derive(Debug, Deserialize)]
pub struct BatchConfiguration {
    pub mods: Vec<ModConfig>,          // List of modification configurations
}

#[derive(Debug, Deserialize)]
pub struct ModConfig {
    pub ba2: Option<bool>,             // Whether this mod uses a BA2 archive
    pub name: String,                  // Name of the BA2 file or mod name
    pub files: Option<Vec<FileConfig>>, // Files within the BA2 to process
    pub config: Option<String>,        // Legacy: Path to the modification config (relative to config file)
}

#[derive(Debug, Deserialize)]
pub struct FileConfig {
    pub path: String,                  // Path to the file (within BA2 if ba2=true)
    pub config: String,                // Path to the modification config
}

#[derive(Debug, Deserialize)]
pub struct ShapeSource {
    source: String,
    shapes: Vec<u16>,
}

#[derive(Debug, Deserialize)]
pub struct SwfModification {
    bounds: Option<Bounds>,
    modifications: Vec<TagModification>,
}

#[derive(Debug, Deserialize)]
struct Bounds {
    x: BoundRange,
    y: BoundRange,
}

#[derive(Debug, Deserialize)]
struct BoundRange {
    min: i32,
    max: i32,
}

#[derive(Debug, Deserialize)]
struct TagModification {
    tag: String,
    id: u16,
    properties: serde_json::Value,
}

fn read_swf_file(path: &str) -> Result<Vec<u8>, String> {
    if is_ba2_path(path) {
        if let Some(ba2_path) = Ba2Path::from_string(path) {
            extract_file_from_ba2(&ba2_path)
        } else {
            Err("Invalid BA2 path format".to_string())
        }
    } else {
        fs::read(path).map_err(|e| format!("Failed to read SWF file: {}", e))
    }
}

#[command]
pub fn convert_swf_to_json(
    _handle: AppHandle,
    swf_path: String,
    json_path: String,
) -> Result<(), String> {
    let swf_data = read_swf_file(&swf_path)?;
    let movie = parse_swf(&swf_data).map_err(|e| format!("Failed to parse SWF: {}", e))?;
    let json = serde_json::to_string_pretty(&movie)
        .map_err(|e| format!("Failed to convert to JSON: {}", e))?;
    fs::write(json_path, json).map_err(|e| format!("Failed to write JSON file: {}", e))?;
    Ok(())
}

#[command]
pub fn apply_json_modifications(
    _handle: AppHandle,
    swf_json_path: String,
    config_json_path: String,
    output_json_path: String,
) -> Result<(), String> {
    println!("Starting JSON modifications process...");
    println!("SWF JSON path: {}", swf_json_path);
    println!("Config JSON path: {}", config_json_path);
    println!("Output JSON path: {}", output_json_path);

    // Read SWF JSON
    let swf_json = fs::read_to_string(&swf_json_path).map_err(|e| {
        println!("Failed to read SWF JSON file '{}': {}", swf_json_path, e);
        format!("Failed to read SWF JSON file: {}", e)
    })?;

    let mut movie: Movie = serde_json::from_str(&swf_json).map_err(|e| {
        println!("Failed to parse SWF JSON file '{}': {}", swf_json_path, e);
        format!("Failed to parse SWF JSON file: {}", e)
    })?;

    // Read config JSON
    let config_json = fs::read_to_string(&config_json_path).map_err(|e| {
        println!("Failed to read config JSON file '{}': {}", config_json_path, e);
        format!("Failed to read config JSON file: {}", e)
    })?;

    println!("Config JSON content: {}", config_json);

    let config: ModificationConfig = serde_json::from_str(&config_json).map_err(|e| {
        println!("Failed to parse config JSON file '{}': {}", config_json_path, e);
        format!("Failed to parse config JSON file: {}", e)
    })?;

    // Apply transparency if specified
    if let Some(transparent_shapes) = &config.transparent {
        println!("Applying transparency...");
        if let Err(e) = apply_transparency(&mut movie, transparent_shapes) {
            println!("Error applying transparency: {}", e);
            return Err(format!("Failed to apply transparency: {}", e));
        }
    }

    // Apply shape replacements if specified
    if let Some(shape_sources) = &config.file {
        println!("Applying shape replacements...");
        if let Err(e) = apply_shape_replacements(&mut movie, shape_sources, &config_json_path) {
            println!("Error applying shape replacements: {}", e);
            return Err(format!("Failed to apply shape replacements: {}", e));
        }
    }

    // Apply other modifications
    println!("Applying SWF modifications...");
    if let Err(e) = apply_modifications(&mut movie, &config.swf) {
        println!("Error applying modifications: {}", e);
        return Err(format!("Failed to apply modifications: {}", e));
    }

    // Write modified JSON
    let modified_json = serde_json::to_string_pretty(&movie).map_err(|e| {
        println!("Failed to serialize modified JSON: {}", e);
        format!("Failed to serialize modified JSON: {}", e)
    })?;

    fs::write(&output_json_path, modified_json).map_err(|e| {
        println!("Failed to write modified JSON file '{}': {}", output_json_path, e);
        format!("Failed to write modified JSON file: {}", e)
    })?;

    println!("JSON modifications completed successfully");
    Ok(())
}

fn apply_shape_replacements(movie: &mut Movie, sources: &[ShapeSource], config_path: &str) -> Result<(), String> {
    // Get the config file's directory
    let config_dir = Path::new(config_path)
        .parent()
        .ok_or_else(|| "Could not determine config file directory".to_string())?;

    for source in sources {
        // Resolve the source path relative to the config file's directory
        let source_path = config_dir.join(&source.source);
        let shapes = parse_shape_source(&source_path)
            .map_err(|e| format!("Failed to parse shape source '{}': {}", source.source, e))?;

        // Replace each specified shape ID with the new shape
        for &shape_id in &source.shapes {
            replace_shape_in_movie(movie, shape_id, shapes.as_slice())
                .map_err(|e| format!("Failed to replace shape {}: {}", shape_id, e))?;
        }
    }
    Ok(())
}

fn point_to_vec2d(from: Point, to: Point) -> swf_types::Vector2D {
    swf_types::Vector2D {
        x: (to.x as f32 - from.x as f32) as i32,
        y: (to.y as f32 - from.y as f32) as i32,
    }
}

fn opacity_to_alpha(opacity: f32) -> u8 {
    (opacity.clamp(0.0, 1.0) * 255.0) as u8
}

fn apply_transform(point: Point, transform: &Transform) -> Point {
    Point::new(
        transform.a * point.x + transform.c * point.y + transform.e,
        transform.b * point.x + transform.d * point.y + transform.f,
    )
}

fn parse_shape_source(path: &Path) -> Result<Vec<Shape>, String> {
    let svg_data = fs::read(path).map_err(|e| format!("Failed to read SVG file: {}", e))?;

    let mut shapes = Vec::new();
    let mut current_shape = Shape {
        initial_styles: ShapeStyles {
            fill: Vec::new(),
            line: Vec::new(),
        },
        records: Vec::new(),
    };

    let xml = String::from_utf8_lossy(&svg_data);
    let mut tokenizer = Tokenizer::from(xml.as_ref());

    let mut in_path = false;
    let mut transform: Option<Transform> = None;
    let mut path_data: Option<String> = None;
    let mut fill_color: Option<Color> = None;
    let mut stroke_color: Option<Color> = None;
    let mut stroke_width = 1.0;
    let mut fill_opacity = 1.0;
    let mut stroke_opacity = 1.0;

    while let Some(token) = tokenizer.next() {
        let token = token.map_err(|e| format!("Failed to parse SVG: {}", e))?;
        match token {
            Token::ElementStart { local, .. } => {
                if local.as_str() == "path" {
                    in_path = true;
                    transform = None;
                    path_data = None;
                    fill_color = None;
                    stroke_color = None;
                    stroke_width = 1.0;
                    fill_opacity = 1.0;
                    stroke_opacity = 1.0;
                }
            }
            Token::Attribute { local, value, .. } if in_path => {
                match local.as_str() {
                    "d" => path_data = Some(value.to_string()),
                    "transform" => {
                        transform = Transform::from_str(value.as_str())
                            .map_err(|e| format!("Failed to parse transform: {}", e))
                            .ok();
                    }
                    "fill" => {
                        fill_color = match value.as_str() {
                            "none" => None,
                            color_str => Color::from_str(color_str)
                                .map_err(|e| format!("Failed to parse fill color: {}", e))
                                .ok(),
                        };
                    }
                    "stroke" => {
                        stroke_color = match value.as_str() {
                            "none" => None,
                            color_str => Color::from_str(color_str)
                                .map_err(|e| format!("Failed to parse stroke color: {}", e))
                                .ok(),
                        };
                    }
                    "stroke-width" => {
                        if let Ok(n) = value.as_str().parse::<f32>() {
                            stroke_width = n;
                        }
                    }
                    "fill-opacity" => {
                        if let Ok(n) = value.as_str().parse::<f32>() {
                            fill_opacity = n;
                        }
                    }
                    "stroke-opacity" => {
                        if let Ok(n) = value.as_str().parse::<f32>() {
                            stroke_opacity = n;
                        }
                    }
                    _ => {}
                }
            }
            Token::ElementEnd { .. } if in_path => {
                in_path = false;

                // Process path data if available
                if let Some(path_str) = path_data.take() {
                    let mut current_pos = Point::new(0.0, 0.0);
                    let path_parser = PathParser::from(path_str.as_str());

                    for segment in path_parser {
                        let segment =
                            segment.map_err(|e| format!("Failed to parse path: {}", e))?;
                        match segment {
                            PathSegment::MoveTo { abs, x, y } => {
                                let point = if abs {
                                    Point::new(x as f64, y as f64)
                                } else {
                                    Point::new(current_pos.x + x as f64, current_pos.y + y as f64)
                                };
                                let transformed_point = transform
                                    .as_ref()
                                    .map(|t| apply_transform(point, t))
                                    .unwrap_or(point);
                                current_shape.records.push(ShapeRecord::StyleChange(
                                    shape_records::StyleChange {
                                        move_to: Some(swf_types::Vector2D {
                                            x: transformed_point.x as i32,
                                            y: transformed_point.y as i32,
                                        }),
                                        left_fill: if fill_color.is_some() { Some(1) } else { None },
                                        right_fill: None,
                                        line_style: if stroke_color.is_some() { Some(1) } else { None },
                                        new_styles: None,
                                    },
                                ));
                                current_pos = transformed_point;
                            }
                            PathSegment::LineTo { abs, x, y } => {
                                let point = if abs {
                                    Point::new(x as f64, y as f64)
                                } else {
                                    Point::new(current_pos.x + x as f64, current_pos.y + y as f64)
                                };
                                let transformed_point = transform
                                    .as_ref()
                                    .map(|t| apply_transform(point, t))
                                    .unwrap_or(point);
                                current_shape.records.push(ShapeRecord::Edge(
                                    shape_records::Edge {
                                        delta: point_to_vec2d(current_pos, transformed_point),
                                        control_delta: None,
                                    },
                                ));
                                current_pos = transformed_point;
                            }
                            PathSegment::CurveTo {
                                abs,
                                x1,
                                y1,
                                x2,
                                y2,
                                x,
                                y,
                            } => {
                                let control = if abs {
                                    Point::new(((x1 + x2) / 2.0) as f64, ((y1 + y2) / 2.0) as f64)
                                } else {
                                    Point::new(
                                        current_pos.x + ((x1 + x2) / 2.0) as f64,
                                        current_pos.y + ((y1 + y2) / 2.0) as f64,
                                    )
                                };
                                let end = if abs {
                                    Point::new(x as f64, y as f64)
                                } else {
                                    Point::new(current_pos.x + x as f64, current_pos.y + y as f64)
                                };
                                let transformed_control = transform
                                    .as_ref()
                                    .map(|t| apply_transform(control, t))
                                    .unwrap_or(control);
                                let transformed_end = transform
                                    .as_ref()
                                    .map(|t| apply_transform(end, t))
                                    .unwrap_or(end);
                                current_shape.records.push(ShapeRecord::Edge(
                                    shape_records::Edge {
                                        delta: point_to_vec2d(current_pos, transformed_end),
                                        control_delta: Some(point_to_vec2d(
                                            current_pos,
                                            transformed_control,
                                        )),
                                    },
                                ));
                                current_pos = transformed_end;
                            }
                            PathSegment::ClosePath { .. } => {
                                let mut first_point = None;
                                for record in &current_shape.records {
                                    if let ShapeRecord::StyleChange(change) = record {
                                        if let Some(pos) = &change.move_to {
                                            first_point =
                                                Some(Point::new(pos.x as f64, pos.y as f64));
                                            break;
                                        }
                                    }
                                }
                                if let Some(start_pos) = first_point {
                                    if (current_pos.x - start_pos.x).abs() > 1.0
                                        || (current_pos.y - start_pos.y).abs() > 1.0
                                    {
                                        current_shape.records.push(ShapeRecord::Edge(
                                            shape_records::Edge {
                                                delta: point_to_vec2d(current_pos, start_pos),
                                                control_delta: None,
                                            },
                                        ));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    // Add styles if specified
                    if let Some(color) = fill_color {
                        current_shape.initial_styles.fill.push(FillStyle::Solid(
                            fill_styles::Solid {
                                color: StraightSRgba8 {
                                    r: color.red,
                                    g: color.green,
                                    b: color.blue,
                                    a: opacity_to_alpha(fill_opacity),
                                },
                            },
                        ));
                    }

                    if let Some(color) = stroke_color {
                        current_shape.initial_styles.line.push(LineStyle {
                            width: stroke_width as u16,
                            start_cap: CapStyle::Round,
                            end_cap: CapStyle::Round,
                            join: JoinStyle::Round,
                            no_h_scale: false,
                            no_v_scale: false,
                            no_close: false,
                            pixel_hinting: false,
                            fill: FillStyle::Solid(fill_styles::Solid {
                                color: StraightSRgba8 {
                                    r: color.red,
                                    g: color.green,
                                    b: color.blue,
                                    a: opacity_to_alpha(stroke_opacity),
                                },
                            }),
                        });
                    }

                    shapes.push(current_shape);
                    current_shape = Shape {
                        initial_styles: ShapeStyles {
                            fill: Vec::new(),
                            line: Vec::new(),
                        },
                        records: Vec::new(),
                    };
                }
            }
            _ => {}
        }
    }

    Ok(shapes)
}

fn replace_shape_in_movie(movie: &mut Movie, shape_id: u16, new_shapes: &[Shape]) -> Result<(), String> {
    println!("Attempting to replace shape ID: {}", shape_id);
    println!("Number of new shapes available: {}", new_shapes.len());

    // Find the shape tag with matching ID
    for tag in &mut movie.tags {
        if let Tag::DefineShape(tag) = tag {
            if tag.id == shape_id {
                println!("Found shape with ID {}", shape_id);
                // Find a matching shape from the new shapes
                if let Some(new_shape) = new_shapes.first() {
                    println!("Original shape styles: {:?}", tag.shape.initial_styles);
                    println!("New shape styles: {:?}", new_shape.initial_styles);

                    // Create a new shape with the original bitmap fills
                    let mut modified_shape = new_shape.clone();

                    // If the new shape has no fills and the original has bitmap fills, preserve them
                    if modified_shape.initial_styles.fill.is_empty() && !tag.shape.initial_styles.fill.is_empty() {
                        // Keep the original bitmap fills
                        modified_shape.initial_styles.fill = tag.shape.initial_styles.fill.clone();

                        // Update all shape records to use the first bitmap fill
                        for record in &mut modified_shape.records {
                            if let ShapeRecord::StyleChange(change) = record {
                                // Set left_fill to 1 to use the first bitmap fill
                                change.left_fill = Some(1);
                                change.right_fill = None;
                            }
                        }
                    }

                    // Calculate new bounds before assigning
                    let new_bounds = calculate_shape_bounds(&modified_shape)?;
                    println!("New shape bounds: {:?}", new_bounds);

                    // Update the shape and bounds
                    tag.shape = modified_shape;
                    tag.bounds = new_bounds;

                    return Ok(());
                }
            }
        }
    }
    Err(format!("Shape with ID {} not found", shape_id))
}

fn calculate_shape_bounds(shape: &Shape) -> Result<Rect, String> {
    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;
    let mut current_x = 0;
    let mut current_y = 0;

    for record in &shape.records {
        match record {
            ShapeRecord::StyleChange(change) => {
                if let Some(move_to) = &change.move_to {
                    current_x = move_to.x;
                    current_y = move_to.y;
                    min_x = min_x.min(current_x);
                    max_x = max_x.max(current_x);
                    min_y = min_y.min(current_y);
                    max_y = max_y.max(current_y);
                }
            }
            ShapeRecord::Edge(edge) => {
                current_x += edge.delta.x;
                current_y += edge.delta.y;
                min_x = min_x.min(current_x);
                max_x = max_x.max(current_x);
                min_y = min_y.min(current_y);
                max_y = max_y.max(current_y);

                if let Some(control) = &edge.control_delta {
                    let control_x = current_x - edge.delta.x + control.x;
                    let control_y = current_y - edge.delta.y + control.y;
                    min_x = min_x.min(control_x);
                    max_x = max_x.max(control_x);
                    min_y = min_y.min(control_y);
                    max_y = max_y.max(control_y);
                }
            }
        }
    }

    if min_x == i32::MAX {
        return Ok(Rect {
            x_min: 0,
            x_max: 0,
            y_min: 0,
            y_max: 0,
        });
    }

    const PADDING: i32 = 10;
    Ok(Rect {
        x_min: min_x - PADDING,
        x_max: max_x + PADDING,
        y_min: min_y - PADDING,
        y_max: max_y + PADDING,
    })
}

fn apply_modifications(movie: &mut Movie, config: &SwfModification) -> Result<(), String> {
    if let Some(bounds) = &config.bounds {
        movie.header.frame_size.x_min = bounds.x.min;
        movie.header.frame_size.x_max = bounds.x.max;
        movie.header.frame_size.y_min = bounds.y.min;
        movie.header.frame_size.y_max = bounds.y.max;
    }

    for modification in &config.modifications {
        apply_tag_modification(movie, modification)?;
    }

    Ok(())
}

fn apply_tag_modification(movie: &mut Movie, modification: &TagModification) -> Result<(), String> {
    for tag in &mut movie.tags {
        match (tag, modification.tag.as_str()) {
            (Tag::DefineBinaryData(tag), "DefineBinaryDataTag") if tag.id == modification.id => {
                if let Some(data) = modification.properties.get("data") {
                    tag.data = serde_json::from_value(data.clone())
                        .map_err(|e| format!("Failed to parse binary data: {}", e))?;
                }
            }
            (Tag::DefineBitmap(tag), "DefineBitmapTag") if tag.id == modification.id => {
                if let Some(data) = modification.properties.get("data") {
                    tag.data = serde_json::from_value(data.clone())
                        .map_err(|e| format!("Failed to parse bitmap data: {}", e))?;
                }
            }
            (Tag::DefineButton(tag), "DefineButtonTag") if tag.id == modification.id => {
                if let Some(records) = modification.properties.get("records") {
                    tag.records = serde_json::from_value(records.clone())
                        .map_err(|e| format!("Failed to parse button records: {}", e))?;
                }
            }
            (Tag::DefineButtonColorTransform(tag), "DefineButtonColorTransformTag")
                if tag.button_id == modification.id =>
            {
                if let Some(transform) = modification.properties.get("transform") {
                    tag.transform = serde_json::from_value(transform.clone())
                        .map_err(|e| format!("Failed to parse color transform: {}", e))?;
                }
            }
            (Tag::DefineButtonSound(tag), "DefineButtonSoundTag")
                if tag.button_id == modification.id =>
            {
                if let Some(over_up_to_idle) = modification.properties.get("overUpToIdle") {
                    tag.over_up_to_idle = serde_json::from_value(over_up_to_idle.clone())
                        .map_err(|e| format!("Failed to parse over_up_to_idle sound: {}", e))?;
                }
                if let Some(idle_to_over_up) = modification.properties.get("idleToOverUp") {
                    tag.idle_to_over_up = serde_json::from_value(idle_to_over_up.clone())
                        .map_err(|e| format!("Failed to parse idle_to_over_up sound: {}", e))?;
                }
                if let Some(over_up_to_over_down) = modification.properties.get("overUpToOverDown")
                {
                    tag.over_up_to_over_down = serde_json::from_value(over_up_to_over_down.clone())
                        .map_err(|e| {
                            format!("Failed to parse over_up_to_over_down sound: {}", e)
                        })?;
                }
                if let Some(over_down_to_over_up) = modification.properties.get("overDownToOverUp")
                {
                    tag.over_down_to_over_up = serde_json::from_value(over_down_to_over_up.clone())
                        .map_err(|e| {
                            format!("Failed to parse over_down_to_over_up sound: {}", e)
                        })?;
                }
            }
            (Tag::DefineDynamicText(tag), "DefineDynamicTextTag") if tag.id == modification.id => {
                if let Some(text) = modification.properties.get("text") {
                    tag.text = serde_json::from_value(text.clone())
                        .map_err(|e| format!("Failed to parse dynamic text: {}", e))?;
                }
            }
            (Tag::DefineFont(tag), "DefineFontTag") if tag.id == modification.id => {
                if let Some(glyphs) = modification.properties.get("glyphs") {
                    tag.glyphs = serde_json::from_value(glyphs.clone())
                        .map_err(|e| format!("Failed to parse font glyphs: {}", e))?;
                }
            }
            (Tag::DefineMorphShape(tag), "DefineMorphShapeTag") if tag.id == modification.id => {
                if let Some(shape) = modification.properties.get("shape") {
                    tag.shape = serde_json::from_value(shape.clone())
                        .map_err(|e| format!("Failed to parse morph shape: {}", e))?;
                }
            }
            (Tag::DefineShape(tag), "DefineShapeTag") if tag.id == modification.id => {
                if let Some(shape) = modification.properties.get("shape") {
                    tag.shape = serde_json::from_value(shape.clone())
                        .map_err(|e| format!("Failed to parse shape: {}", e))?;
                } else {
                    if let Some(bounds) = modification.properties.get("bounds") {
                        tag.bounds = serde_json::from_value(bounds.clone())
                            .map_err(|e| format!("Failed to parse shape bounds: {}", e))?;
                    }
                    if let Some(records) = modification.properties.get("records") {
                        tag.shape.records = serde_json::from_value(records.clone())
                            .map_err(|e| format!("Failed to parse shape records: {}", e))?;
                    }
                    if let Some(styles) = modification.properties.get("styles") {
                        tag.shape.initial_styles = serde_json::from_value(styles.clone())
                            .map_err(|e| format!("Failed to parse shape styles: {}", e))?;
                    } else {
                        if let Some(fill_styles) = modification.properties.get("fillStyles") {
                            tag.shape.initial_styles.fill =
                                serde_json::from_value(fill_styles.clone())
                                    .map_err(|e| format!("Failed to parse fill styles: {}", e))?;
                        }
                        if let Some(line_styles) = modification.properties.get("lineStyles") {
                            tag.shape.initial_styles.line =
                                serde_json::from_value(line_styles.clone())
                                    .map_err(|e| format!("Failed to parse line styles: {}", e))?;
                        }
                    }
                }
            }
            (Tag::DefineSprite(tag), "DefineSpriteTag") if tag.id == modification.id => {
                if let Some(tags) = modification.properties.get("tags") {
                    tag.tags = serde_json::from_value(tags.clone())
                        .map_err(|e| format!("Failed to parse sprite tags: {}", e))?;
                }
            }
            (Tag::DefineText(tag), "DefineTextTag") if tag.id == modification.id => {
                if let Some(records) = modification.properties.get("records") {
                    tag.records = serde_json::from_value(records.clone())
                        .map_err(|e| format!("Failed to parse text records: {}", e))?;
                }
            }

            (Tag::DoAbc(tag), "DoAbcTag") => {
                if let Some(data) = modification.properties.get("data") {
                    tag.data = serde_json::from_value(data.clone())
                        .map_err(|e| format!("Failed to parse ABC data: {}", e))?;
                }
            }
            (Tag::DoAction(tag), "DoActionTag") => {
                if let Some(actions) = modification.properties.get("actions") {
                    tag.actions = serde_json::from_value(actions.clone())
                        .map_err(|e| format!("Failed to parse actions: {}", e))?;
                }
            }
            (Tag::FileAttributes(tag), "FileAttributesTag") => {
                if let Some(props) = modification.properties.as_object() {
                    if let Some(as3) = props.get("actionScript3") {
                        tag.use_as3 = as3.as_bool().unwrap_or(false);
                    }
                    if let Some(metadata) = props.get("hasMetadata") {
                        tag.has_metadata = metadata.as_bool().unwrap_or(false);
                    }
                    if let Some(network) = props.get("useNetwork") {
                        tag.use_network = network.as_bool().unwrap_or(false);
                    }
                    if let Some(gpu) = props.get("useGPU") {
                        tag.use_direct_blit = gpu.as_bool().unwrap_or(false);
                    }
                }
            }
            (Tag::FrameLabel(tag), "FrameLabelTag") => {
                if let Some(name) = modification.properties.get("name") {
                    tag.name = serde_json::from_value(name.clone())
                        .map_err(|e| format!("Failed to parse frame label: {}", e))?;
                }
            }
            (Tag::PlaceObject(tag), "PlaceObjectTag") => {
                if let Some(matrix) = modification.properties.get("matrix") {
                    tag.matrix = serde_json::from_value(matrix.clone())
                        .map_err(|e| format!("Failed to parse matrix: {}", e))?;
                }
                if let Some(color_transform) = modification.properties.get("colorTransform") {
                    tag.color_transform = serde_json::from_value(color_transform.clone())
                        .map_err(|e| format!("Failed to parse color transform: {}", e))?;
                }
            }
            (Tag::RemoveObject(tag), "RemoveObjectTag") => {
                if let Some(depth) = modification.properties.get("depth") {
                    tag.depth = serde_json::from_value(depth.clone())
                        .map_err(|e| format!("Failed to parse depth: {}", e))?;
                }
            }
            (Tag::SetBackgroundColor(tag), "SetBackgroundColorTag") => {
                if let Some(color) = modification.properties.get("backgroundColor") {
                    let rgba: StraightSRgba8 = serde_json::from_value(color.clone())
                        .map_err(|e| format!("Failed to parse color: {}", e))?;
                    tag.color = SRgb8 {
                        r: rgba.r,
                        g: rgba.g,
                        b: rgba.b,
                    };
                }
            }
            (Tag::StartSound(tag), "StartSoundTag") => {
                if let Some(sound_info) = modification.properties.get("soundInfo") {
                    tag.sound_info = serde_json::from_value(sound_info.clone())
                        .map_err(|e| format!("Failed to parse sound info: {}", e))?;
                }
            }
            (Tag::SymbolClass(tag), "SymbolClassTag") => {
                if let Some(symbols) = modification.properties.get("symbols") {
                    tag.symbols = serde_json::from_value(symbols.clone())
                        .map_err(|e| format!("Failed to parse symbols: {}", e))?;
                }
            }

            (Tag::DefineSceneAndFrameLabelData(tag), "DefineSceneAndFrameLabelDataTag") => {
                if let Some(scenes) = modification.properties.get("scenes") {
                    tag.scenes = serde_json::from_value(scenes.clone())
                        .map_err(|e| format!("Failed to parse scenes: {}", e))?;
                }
                if let Some(labels) = modification.properties.get("labels") {
                    tag.labels = serde_json::from_value(labels.clone())
                        .map_err(|e| format!("Failed to parse labels: {}", e))?;
                }
            }
            _ => continue,
        }
    }
    Ok(())
}

#[command]
pub fn convert_json_to_swf(
    _handle: AppHandle,
    json_path: String,
    swf_path: String,
) -> Result<(), String> {
    println!("Starting SWF conversion process...");
    println!("Input JSON: {}", json_path);
    println!("Output SWF: {}", swf_path);

    // Read the modified JSON
    println!("Reading modified JSON file...");
    let json_data = fs::read_to_string(&json_path).map_err(|e| {
        println!("Failed to read JSON file: {}", e);
        format!("Failed to read JSON file '{}': {}", json_path, e)
    })?;

    // Parse JSON to Movie
    println!("Parsing JSON to Movie structure...");
    let movie: Movie = serde_json::from_str(&json_data).map_err(|e| {
        println!("Failed to parse JSON to Movie: {}", e);
        format!("Failed to parse JSON file '{}': {}", json_path, e)
    })?;

    // Convert Movie to binary SWF
    println!("Converting Movie to binary SWF...");
    let swf_data = emit_swf(&movie, swf_types::CompressionMethod::None).map_err(|e| {
        println!("Failed to emit SWF: {}", e);
        format!("Failed to create SWF from JSON '{}': {}", json_path, e)
    })?;

    // Write the SWF file directly
    println!("Writing SWF file to: {}", swf_path);
    fs::write(&swf_path, swf_data).map_err(|e| {
        println!("Failed to write SWF file: {}", e);
        format!("Failed to write SWF file '{}': {}", swf_path, e)
    })?;

    println!("SWF conversion completed successfully");
    Ok(())
}

#[command]
pub fn get_file_size(_handle: AppHandle, path: String) -> Result<u64, String> {
    let metadata = fs::metadata(path.clone()).map_err(|e| {
        println!("Failed to get metadata for file '{}': {}", path, e);
        format!("Failed to get file metadata: {}", e)
    })?;
    Ok(metadata.len())
}

fn apply_transparency(movie: &mut Movie, shape_ids: &[u16]) -> Result<(), String> {
    println!("Making shapes transparent...");

    // Ensure we have a high enough SWF version for alpha support
    if movie.header.swf_version < 8 {
        movie.header.swf_version = 8;
    }

    // First pass: Fix any dynamic text tags to ensure they have both font_class and font_size
    for tag in &mut movie.tags {
        if let Tag::DefineDynamicText(text) = tag {
            if text.font_class.is_some() && text.font_size.is_none() {
                // If we have a font class but no size, set a default size
                text.font_size = Some(12);
            }
        }
    }

    // Second pass: Handle shape transparency
    for &shape_id in shape_ids {
        println!("Making shape {} transparent", shape_id);

        // Find and modify the shape tag
        for i in 0..movie.tags.len() {
            if let Tag::DefineShape(shape_tag) = &movie.tags[i] {
                if shape_tag.id == shape_id {
                    println!("Found shape {} - converting to DefineShape3 and making transparent", shape_id);

                    // Create a new shape with transparent fills
                    let new_shape = Shape {
                        initial_styles: ShapeStyles {
                            fill: vec![
                                FillStyle::Solid(fill_styles::Solid {
                                    color: StraightSRgba8 {
                                        r: 0,
                                        g: 0,
                                        b: 0,
                                        a: 0,  // Alpha 0 will force Shape3
                                    },
                                }),
                                FillStyle::Solid(fill_styles::Solid {
                                    color: StraightSRgba8 {
                                        r: 0,
                                        g: 0,
                                        b: 0,
                                        a: 0,  // Alpha 0 will force Shape3
                                    },
                                }),
                            ],
                            line: Vec::new(),
                        },
                        records: shape_tag.shape.records.clone(),
                    };

                    // Create a new DefineShape tag
                    let new_tag = Tag::DefineShape(swf_types::tags::DefineShape {
                        id: shape_id,
                        bounds: shape_tag.bounds.clone(),
                        edge_bounds: None,  // Don't set edge_bounds to avoid forcing Shape4
                        has_fill_winding: false,
                        has_non_scaling_strokes: false,
                        has_scaling_strokes: false,
                        shape: new_shape,
                    });

                    // Replace the old tag with the new one
                    movie.tags[i] = new_tag;
                    println!("Successfully converted shape {} to DefineShape3 with transparency", shape_id);
                    break;
                }
            }
        }
    }

    Ok(())
}

#[command]
pub fn batch_process_swf(
    _handle: AppHandle,
    config: BatchProcessConfig,
) -> Result<Vec<String>, String> {
    println!("Starting batch SWF processing...");
    let mut processed_files = Vec::new();

    // Read and parse the batch configuration
    let config_json = fs::read_to_string(&config.config_file).map_err(|e| {
        format!("Failed to read batch config file '{}': {}", config.config_file, e)
    })?;

    let batch_config: BatchConfiguration = serde_json::from_str(&config_json).map_err(|e| {
        format!("Failed to parse batch config file '{}': {}", config.config_file, e)
    })?;

    // Get the config file's directory for resolving relative paths
    let config_dir = Path::new(&config.config_file)
        .parent()
        .ok_or_else(|| "Could not determine config file directory".to_string())?;

    // Process each mod configuration
    for mod_config in &batch_config.mods {
        // Handle BA2 archives
        if mod_config.ba2 == Some(true) {
            // Get the BA2 path from user selection or config
            let ba2_path = config.ba2_path.as_ref()
                .ok_or_else(|| "BA2 path not provided for BA2 mod".to_string())?;

            // Process each file in the BA2
            if let Some(files) = &mod_config.files {
                for file_config in files {
                    // Construct the full BA2 path (ba2_path//internal/path)
                    let full_path = format!("{}//{}",
                        ba2_path,
                        file_config.path.trim_start_matches("//")
                    );

                    // Get the file name for output
                    let file_name = Path::new(&file_config.path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .ok_or_else(|| format!("Invalid file path in BA2: {}", file_config.path))?;

                    // Setup paths
                    let temp_json_path = PathBuf::from(&config.output_directory)
                        .join(format!("{}.temp.json", file_name));
                    let output_path = PathBuf::from(&config.output_directory)
                        .join(file_name);
                    let config_path = config_dir.join(&file_config.config);

                    println!("Processing BA2 file: {} with config: {}", full_path, config_path.display());

                    // Process the file
                    process_single_file(
                        _handle.clone(),
                        &full_path,
                        &temp_json_path,
                        &output_path,
                        &config_path,
                    )?;

                    processed_files.push(output_path.to_string_lossy().to_string());
                }
            }
        } else {
            // Legacy non-BA2 handling - single file with config
            if let Some(config_path) = &mod_config.config {
                // For non-BA2 mods, the name field contains the target SWF file name
                let swf_path = mod_config.name.clone();
                let file_name = Path::new(&swf_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| format!("Invalid SWF file path: {}", swf_path))?;

                // Setup paths
                let temp_json_path = PathBuf::from(&config.output_directory)
                    .join(format!("{}.temp.json", file_name));
                let output_path = PathBuf::from(&config.output_directory)
                    .join(file_name);
                let config_path = config_dir.join(config_path);

                println!("Processing file: {} with config: {}", swf_path, config_path.display());

                // Process the file
                process_single_file(
                    _handle.clone(),
                    &swf_path,
                    &temp_json_path,
                    &output_path,
                    &config_path,
                )?;

                processed_files.push(output_path.to_string_lossy().to_string());
            }
        }
    }

    println!("Batch processing completed successfully");
    Ok(processed_files)
}

// Helper function to process a single file (used by both BA2 and non-BA2 paths)
fn process_single_file(
    handle: AppHandle,
    input_path: &str,
    temp_json_path: &Path,
    output_path: &Path,
    config_path: &Path,
) -> Result<(), String> {
    // Convert SWF to JSON
    convert_swf_to_json(
        handle.clone(),
        input_path.to_string(),
        temp_json_path.to_string_lossy().to_string(),
    )?;

    // Apply modifications
    apply_json_modifications(
        handle.clone(),
        temp_json_path.to_string_lossy().to_string(),
        config_path.to_string_lossy().to_string(),
        temp_json_path.to_string_lossy().to_string(),
    )?;

    // Convert back to SWF
    convert_json_to_swf(
        handle.clone(),
        temp_json_path.to_string_lossy().to_string(),
        output_path.to_string_lossy().to_string(),
    )?;

    // Clean up temporary JSON file
    if let Err(e) = fs::remove_file(temp_json_path) {
        println!("Warning: Failed to clean up temporary file '{}': {}", temp_json_path.display(), e);
    }

    Ok(())
}

#[command]
pub fn read_file_to_string(_handle: AppHandle, path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| {
        println!("Failed to read file '{}': {}", path, e);
        format!("Failed to read file: {}", e)
    })
}
