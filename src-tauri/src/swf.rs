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
    Shape, ShapeRecord, ShapeStyles, StraightSRgba8, Tag, text, tags,
};
use tauri::{command, AppHandle};
use xmlparser::{Token, Tokenizer};
use crate::ba2::{Ba2Path, extract_file_from_ba2, is_ba2_path};
use std::process::Command;
use tempfile::TempDir;

const SWF_SCALE: f32 = 20.0;  // SWF uses 20 twips per pixel, whereas SVG uses 1px per pixel

#[derive(Debug, Deserialize)]
pub struct ModificationConfig {
    pub file: Option<Vec<ShapeSource>>,
    pub transparent: Option<Vec<u16>>,  // Shape IDs to make transparent
    pub actionscript: Option<Vec<ActionScriptPatch>>,  // New field for ActionScript patches
    pub swf: SwfModification,
    pub new_elements: Option<NewElements>,  // New field for adding elements
    pub remove_elements: Option<RemoveElements>,  // New field for removing elements
}

#[derive(Debug, Deserialize)]
pub struct BatchProcessConfig {
    pub config_file: String,           // Path to the main configuration file
    pub output_directory: String,      // Directory to save processed files
    pub ba2_path: Option<String>,      // User-selected BA2 file path (if using BA2)
    pub swf_mappings: Vec<SwfMapping>, // Mappings from mod names to SWF file paths
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
    new_elements: Option<NewElements>,
    remove_elements: Option<RemoveElements>,
}

#[derive(Debug, Deserialize)]
pub struct Bounds {
    pub x: BoundRange,
    pub y: BoundRange,
}

#[derive(Debug, Deserialize)]
pub struct BoundRange {
    pub min: i32,
    pub max: i32,
}

#[derive(Debug, Deserialize)]
struct TagModification {
    tag: String,
    id: u16,
    properties: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct SwfMapping {
    pub mod_name: String,
    pub swf_path: String,
}

#[derive(Debug, Deserialize)]
pub struct NewElements {
    pub shapes: Option<Vec<NewShape>>,
    pub sprites: Option<Vec<NewSprite>>,
    pub texts: Option<Vec<NewText>>,
    pub bitmaps: Option<Vec<NewBitmap>>,
    pub buttons: Option<Vec<NewButton>>,
    pub scenes: Option<Vec<NewScene>>,
}

#[derive(Debug, Deserialize)]
pub struct NewShape {
    pub source: String,           // Path to SVG source
    pub id: Option<u16>,         // Optional ID (if not provided, will auto-generate)
    pub bounds: Option<Bounds>,   // Optional bounds override
}

#[derive(Debug, Deserialize)]
pub struct NewSprite {
    pub id: Option<u16>,
    pub frame_count: u16,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
pub struct NewText {
    pub id: Option<u16>,
    pub text: String,
    pub bounds: Bounds,
    pub font_class: String,
    pub font_size: u16,
    pub color: Option<StraightSRgba8>,
    pub word_wrap: bool,
    pub multiline: bool,
    pub readonly: bool,
    pub no_select: bool,
    pub html: bool,
    pub use_outlines: bool,
    pub align: text::TextAlignment,
    pub margin_left: u16,
    pub margin_right: u16,
    pub indent: u16,
    pub leading: i16,
}

#[derive(Debug, Deserialize)]
pub struct RemoveElements {
    pub shapes: Option<Vec<u16>>,      // Shape IDs to remove
    pub sprites: Option<Vec<u16>>,     // Sprite IDs to remove
    pub texts: Option<Vec<u16>>,       // Text IDs to remove
    pub buttons: Option<Vec<u16>>,     // Button IDs to remove
    pub bitmaps: Option<Vec<u16>>,     // Bitmap IDs to remove
    pub frames: Option<Vec<String>>,   // Frame labels to remove
    pub scenes: Option<Vec<String>>,   // Scene names to remove
}

#[derive(Debug, Deserialize)]
pub struct NewBitmap {
    pub id: Option<u16>,
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct NewButton {
    pub states: Vec<swf_types::tags::DefineButton>,
}

#[derive(Debug, Deserialize)]
pub struct NewScene {
    pub name: String,
    pub offset: u32,
}

#[derive(Debug, Deserialize)]
pub struct ActionScriptPatch {
    pub source_file: String,           // Path to the ActionScript source file
    pub insert_mode: ActionScriptInsertMode,  // How to insert the script
    pub class_name: Option<String>,    // Optional class name for replacement
    pub package_name: Option<String>,  // Optional package name
    pub symbol_bindings: Option<Vec<SymbolBinding>>,  // Optional symbol class bindings
}

#[derive(Debug, Deserialize)]
pub struct SymbolBinding {
    pub symbol_id: u16,                // The ID of the symbol (shape, sprite, etc.)
    pub class_name: String,            // The fully qualified class name to bind to
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(PartialEq)]
pub enum ActionScriptInsertMode {
    Add,     // Add the script as a new DoABC tag
    Replace  // Replace an existing script with matching class name
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

    // Apply ActionScript patches if specified
    if let Some(actionscript_patches) = &config.actionscript {
        println!("Applying ActionScript patches...");
        if let Err(e) = apply_actionscript_patches(&mut movie, actionscript_patches, &config_json_path, _handle) {
            println!("Error applying ActionScript patches: {}", e);
            return Err(format!("Failed to apply ActionScript patches: {}", e));
        }
    }

    // Apply other modifications
    println!("Applying SWF modifications...");
    if let Err(e) = apply_modifications(&mut movie, &config.swf, Path::new(&config_json_path)) {
        println!("Error applying modifications: {}", e);
        return Err(format!("Failed to apply modifications: {}", e));
    }

    // Handle new elements from the root config if present
    if let Some(new_elements) = &config.new_elements {
        println!("Applying new elements from root config...");
        add_new_elements(&mut movie, new_elements)?;
    }

    // Handle element removal from both root config and swf config
    if let Some(remove_elements) = &config.remove_elements {
        println!("Applying element removal from root config...");
        remove_swf_elements(&mut movie, remove_elements)?;
    }
    if let Some(remove_elements) = &config.swf.remove_elements {
        println!("Applying element removal from swf config...");
        remove_swf_elements(&mut movie, remove_elements)?;
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
        x: ((to.x * SWF_SCALE as f64) as i32 - (from.x * SWF_SCALE as f64) as i32),
        y: ((to.y * SWF_SCALE as f64) as i32 - (from.y * SWF_SCALE as f64) as i32),
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
    println!("Starting to parse SVG file: {}", path.display());
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
    let mut group_transform: Option<Transform> = None;
    let mut path_transform: Option<Transform> = None;
    let mut path_data: Option<String> = None;
    let mut fill_color: Option<Color> = None;
    let mut stroke_color: Option<Color> = None;
    let mut stroke_width = 1.0;
    let mut fill_opacity = 1.0;
    let mut stroke_opacity = 1.0;
    let mut path_count = 0;
    let mut current_fill_style_index = 0;
    let mut current_line_style_index = 0;

    println!("Starting XML parsing");
    while let Some(token) = tokenizer.next() {
        let token = token.map_err(|e| format!("Failed to parse SVG: {}", e))?;
        match token {
            Token::ElementStart { local, .. } => {
                if local.as_str() == "path" {
                    path_count += 1;
                    println!("Found path #{}", path_count);
                    in_path = true;
                    path_transform = None;
                    path_data = None;
                    fill_color = None;
                    stroke_color = None;
                    stroke_width = 1.0;
                    fill_opacity = 1.0;
                    stroke_opacity = 1.0;
                } else if local.as_str() == "g" {
                    println!("Found group element");
                }
            }
            Token::Attribute { local, value, .. } => {
                match local.as_str() {
                    "transform" => {
                        println!("Found transform: {}", value.as_str());
                        let transform = Transform::from_str(value.as_str()).ok();
                        if in_path {
                            path_transform = transform;
                        } else {
                            group_transform = transform;
                        }
                    }
                    "d" if in_path => {
                        println!("Found path data");
                        path_data = Some(value.as_str().to_string());
                    }
                    "fill" if in_path => {
                        println!("Found fill color: {}", value.as_str());
                        fill_color = Color::from_str(value.as_str()).ok();
                    }
                    "fill-opacity" => {
                        if let Ok(n) = value.as_str().parse::<f32>() {
                            fill_opacity = n;
                        }
                    }
                    "stroke" => {
                        stroke_color = match value.as_str() {
                            "none" => None,
                            color_str => Color::from_str(color_str).ok(),
                        };
                    }
                    "stroke-width" => {
                        if let Ok(n) = value.as_str().parse::<f32>() {
                            stroke_width = n;
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
                    println!("Processing path with {} characters", path_str.len());
                    let mut current_pos = Point::new(0.0, 0.0);
                    let path_parser = PathParser::from(path_str.as_str());

                    // Add new styles if needed for this path
                    let mut new_styles = ShapeStyles {
                        fill: Vec::new(),
                        line: Vec::new(),
                    };

                    // Add fill style if one is defined
                    if let Some(color) = fill_color {
                        current_fill_style_index += 1;
                        new_styles.fill.push(FillStyle::Solid(fill_styles::Solid {
                            color: StraightSRgba8 {
                                r: color.red,
                                g: color.green,
                                b: color.blue,
                                a: opacity_to_alpha(fill_opacity),
                            },
                        }));
                    }

                    // Add line style if stroke is defined
                    if let Some(stroke) = stroke_color {
                        current_line_style_index += 1;
                        new_styles.line.push(LineStyle {
                            width: (stroke_width * SWF_SCALE) as u16,
                            start_cap: CapStyle::Round,
                            end_cap: CapStyle::Round,
                            join: JoinStyle::Round,
                            no_h_scale: false,
                            no_v_scale: false,
                            no_close: false,
                            pixel_hinting: false,
                            fill: FillStyle::Solid(fill_styles::Solid {
                                color: StraightSRgba8 {
                                    r: stroke.red,
                                    g: stroke.green,
                                    b: stroke.blue,
                                    a: opacity_to_alpha(stroke_opacity),
                                },
                            }),
                        });
                    }

                    // Only add the style change record if we have new styles
                    if !new_styles.fill.is_empty() || !new_styles.line.is_empty() {
                        current_shape.records.push(ShapeRecord::StyleChange(
                            shape_records::StyleChange {
                                move_to: None,
                                left_fill: if !new_styles.fill.is_empty() { Some(current_fill_style_index) } else { None },
                                right_fill: None,
                                line_style: if !new_styles.line.is_empty() { Some(current_line_style_index) } else { None },
                                new_styles: Some(new_styles),
                            },
                        ));
                    }

                    // Process path segments
                    let mut last_control_point: Option<Point> = None;
                    for segment in path_parser {
                        let segment = segment.map_err(|e| format!("Failed to parse path: {}", e))?;
                        match segment {
                            PathSegment::MoveTo { abs, x, y } => {
                                let point = if abs {
                                    Point::new(x as f64, y as f64)
                                } else {
                                    Point::new(current_pos.x + x as f64, current_pos.y + y as f64)
                                };

                                let transformed_point = path_transform
                                    .as_ref()
                                    .map(|t| apply_transform(point, t))
                                    .unwrap_or(point);
                                let transformed_point = group_transform
                                    .as_ref()
                                    .map(|t| apply_transform(transformed_point, t))
                                    .unwrap_or(transformed_point);

                                current_shape.records.push(ShapeRecord::StyleChange(
                                    shape_records::StyleChange {
                                        move_to: Some(swf_types::Vector2D {
                                            x: (transformed_point.x * SWF_SCALE as f64) as i32,
                                            y: (transformed_point.y * SWF_SCALE as f64) as i32,
                                        }),
                                        right_fill: None,
                                        left_fill: None,
                                        line_style: None,
                                        new_styles: None,
                                    },
                                ));

                                current_pos = transformed_point;
                                last_control_point = None;
                            },
                            PathSegment::LineTo { abs, x, y } => {
                                let point = if abs {
                                    Point::new(x as f64, y as f64)
                                } else {
                                    Point::new(current_pos.x + x as f64, current_pos.y + y as f64)
                                };

                                let transformed_point = path_transform
                                    .as_ref()
                                    .map(|t| apply_transform(point, t))
                                    .unwrap_or(point);
                                let transformed_point = group_transform
                                    .as_ref()
                                    .map(|t| apply_transform(transformed_point, t))
                                    .unwrap_or(transformed_point);

                                current_shape.records.push(ShapeRecord::Edge(
                                    shape_records::Edge {
                                        delta: point_to_vec2d(current_pos, transformed_point),
                                        control_delta: None,
                                    },
                                ));

                                current_pos = transformed_point;
                                last_control_point = None;
                            },
                            PathSegment::HorizontalLineTo { abs, x } => {
                                let point = if abs {
                                    Point::new(x as f64, current_pos.y)
                                } else {
                                    Point::new(current_pos.x + x as f64, current_pos.y)
                                };

                                let transformed_point = path_transform
                                    .as_ref()
                                    .map(|t| apply_transform(point, t))
                                    .unwrap_or(point);
                                let transformed_point = group_transform
                                    .as_ref()
                                    .map(|t| apply_transform(transformed_point, t))
                                    .unwrap_or(transformed_point);

                                current_shape.records.push(ShapeRecord::Edge(
                                    shape_records::Edge {
                                        delta: point_to_vec2d(current_pos, transformed_point),
                                        control_delta: None,
                                    },
                                ));

                                current_pos = transformed_point;
                                last_control_point = None;
                            },
                            PathSegment::VerticalLineTo { abs, y } => {
                                let point = if abs {
                                    Point::new(current_pos.x, y as f64)
                                } else {
                                    Point::new(current_pos.x, current_pos.y + y as f64)
                                };

                                let transformed_point = path_transform
                                    .as_ref()
                                    .map(|t| apply_transform(point, t))
                                    .unwrap_or(point);
                                let transformed_point = group_transform
                                    .as_ref()
                                    .map(|t| apply_transform(transformed_point, t))
                                    .unwrap_or(transformed_point);

                                current_shape.records.push(ShapeRecord::Edge(
                                    shape_records::Edge {
                                        delta: point_to_vec2d(current_pos, transformed_point),
                                        control_delta: None,
                                    },
                                ));

                                current_pos = transformed_point;
                                last_control_point = None;
                            },
                            PathSegment::CurveTo { abs, x1, y1, x2, y2, x, y } => {
                                let control1 = if abs {
                                    Point::new(x1 as f64, y1 as f64)
                                } else {
                                    Point::new(current_pos.x + x1 as f64, current_pos.y + y1 as f64)
                                };
                                let control2 = if abs {
                                    Point::new(x2 as f64, y2 as f64)
                                } else {
                                    Point::new(current_pos.x + x2 as f64, current_pos.y + y2 as f64)
                                };
                                let end = if abs {
                                    Point::new(x as f64, y as f64)
                                } else {
                                    Point::new(current_pos.x + x as f64, current_pos.y + y as f64)
                                };

                                // Transform all points
                                let transformed_control1 = path_transform
                                    .as_ref()
                                    .map(|t| apply_transform(control1, t))
                                    .unwrap_or(control1);
                                let transformed_control1 = group_transform
                                    .as_ref()
                                    .map(|t| apply_transform(transformed_control1, t))
                                    .unwrap_or(transformed_control1);

                                let transformed_control2 = path_transform
                                    .as_ref()
                                    .map(|t| apply_transform(control2, t))
                                    .unwrap_or(control2);
                                let transformed_control2 = group_transform
                                    .as_ref()
                                    .map(|t| apply_transform(transformed_control2, t))
                                    .unwrap_or(transformed_control2);

                                let transformed_end = path_transform
                                    .as_ref()
                                    .map(|t| apply_transform(end, t))
                                    .unwrap_or(end);
                                let transformed_end = group_transform
                                    .as_ref()
                                    .map(|t| apply_transform(transformed_end, t))
                                    .unwrap_or(transformed_end);

                                // Convert cubic to two quadratic curves
                                let mid = Point::new(
                                    (transformed_control1.x + transformed_control2.x) / 2.0,
                                    (transformed_control1.y + transformed_control2.y) / 2.0
                                );

                                // First quadratic curve
                                current_shape.records.push(ShapeRecord::Edge(
                                    shape_records::Edge {
                                        delta: point_to_vec2d(current_pos, mid),
                                        control_delta: Some(point_to_vec2d(current_pos, transformed_control1)),
                                    },
                                ));

                                // Second quadratic curve
                                current_shape.records.push(ShapeRecord::Edge(
                                    shape_records::Edge {
                                        delta: point_to_vec2d(mid, transformed_end),
                                        control_delta: Some(point_to_vec2d(mid, transformed_control2)),
                                    },
                                ));

                                current_pos = transformed_end;
                                last_control_point = Some(transformed_control2);
                            },
                            PathSegment::SmoothCurveTo { abs, x2, y2, x, y } => {
                                let control1 = match last_control_point {
                                    Some(last_ctrl) => Point::new(
                                        2.0 * current_pos.x - last_ctrl.x,
                                        2.0 * current_pos.y - last_ctrl.y
                                    ),
                                    None => current_pos
                                };

                                let control2 = if abs {
                                    Point::new(x2 as f64, y2 as f64)
                                } else {
                                    Point::new(current_pos.x + x2 as f64, current_pos.y + y2 as f64)
                                };
                                let end = if abs {
                                    Point::new(x as f64, y as f64)
                                } else {
                                    Point::new(current_pos.x + x as f64, current_pos.y + y as f64)
                                };

                                // Transform all points
                                let transformed_control1 = path_transform
                                    .as_ref()
                                    .map(|t| apply_transform(control1, t))
                                    .unwrap_or(control1);
                                let transformed_control1 = group_transform
                                    .as_ref()
                                    .map(|t| apply_transform(transformed_control1, t))
                                    .unwrap_or(transformed_control1);

                                let transformed_control2 = path_transform
                                    .as_ref()
                                    .map(|t| apply_transform(control2, t))
                                    .unwrap_or(control2);
                                let transformed_control2 = group_transform
                                    .as_ref()
                                    .map(|t| apply_transform(transformed_control2, t))
                                    .unwrap_or(transformed_control2);

                                let transformed_end = path_transform
                                    .as_ref()
                                    .map(|t| apply_transform(end, t))
                                    .unwrap_or(end);
                                let transformed_end = group_transform
                                    .as_ref()
                                    .map(|t| apply_transform(transformed_end, t))
                                    .unwrap_or(transformed_end);

                                // Convert cubic to two quadratic curves
                                let mid = Point::new(
                                    (transformed_control1.x + transformed_control2.x) / 2.0,
                                    (transformed_control1.y + transformed_control2.y) / 2.0
                                );

                                // First quadratic curve
                                current_shape.records.push(ShapeRecord::Edge(
                                    shape_records::Edge {
                                        delta: point_to_vec2d(current_pos, mid),
                                        control_delta: Some(point_to_vec2d(current_pos, transformed_control1)),
                                    },
                                ));

                                // Second quadratic curve
                                current_shape.records.push(ShapeRecord::Edge(
                                    shape_records::Edge {
                                        delta: point_to_vec2d(mid, transformed_end),
                                        control_delta: Some(point_to_vec2d(mid, transformed_control2)),
                                    },
                                ));

                                current_pos = transformed_end;
                                last_control_point = Some(transformed_control2);
                            },
                            PathSegment::Quadratic { abs, x1, y1, x, y } => {
                                let control = if abs {
                                    Point::new(x1 as f64, y1 as f64)
                                } else {
                                    Point::new(current_pos.x + x1 as f64, current_pos.y + y1 as f64)
                                };
                                let end = if abs {
                                    Point::new(x as f64, y as f64)
                                } else {
                                    Point::new(current_pos.x + x as f64, current_pos.y + y as f64)
                                };

                                // Transform points
                                let transformed_control = path_transform
                                    .as_ref()
                                    .map(|t| apply_transform(control, t))
                                    .unwrap_or(control);
                                let transformed_control = group_transform
                                    .as_ref()
                                    .map(|t| apply_transform(transformed_control, t))
                                    .unwrap_or(transformed_control);

                                let transformed_end = path_transform
                                    .as_ref()
                                    .map(|t| apply_transform(end, t))
                                    .unwrap_or(end);
                                let transformed_end = group_transform
                                    .as_ref()
                                    .map(|t| apply_transform(transformed_end, t))
                                    .unwrap_or(transformed_end);

                                current_shape.records.push(ShapeRecord::Edge(
                                    shape_records::Edge {
                                        delta: point_to_vec2d(current_pos, transformed_end),
                                        control_delta: Some(point_to_vec2d(current_pos, transformed_control)),
                                    },
                                ));

                                current_pos = transformed_end;
                                last_control_point = Some(transformed_control);
                            },
                            PathSegment::SmoothQuadratic { abs, x, y } => {
                                let control = match last_control_point {
                                    Some(last_ctrl) => Point::new(
                                        2.0 * current_pos.x - last_ctrl.x,
                                        2.0 * current_pos.y - last_ctrl.y
                                    ),
                                    None => current_pos
                                };
                                let end = if abs {
                                    Point::new(x as f64, y as f64)
                                } else {
                                    Point::new(current_pos.x + x as f64, current_pos.y + y as f64)
                                };

                                // Transform points
                                let transformed_control = path_transform
                                    .as_ref()
                                    .map(|t| apply_transform(control, t))
                                    .unwrap_or(control);
                                let transformed_control = group_transform
                                    .as_ref()
                                    .map(|t| apply_transform(transformed_control, t))
                                    .unwrap_or(transformed_control);

                                let transformed_end = path_transform
                                    .as_ref()
                                    .map(|t| apply_transform(end, t))
                                    .unwrap_or(end);
                                let transformed_end = group_transform
                                    .as_ref()
                                    .map(|t| apply_transform(transformed_end, t))
                                    .unwrap_or(transformed_end);

                                current_shape.records.push(ShapeRecord::Edge(
                                    shape_records::Edge {
                                        delta: point_to_vec2d(current_pos, transformed_end),
                                        control_delta: Some(point_to_vec2d(current_pos, transformed_control)),
                                    },
                                ));

                                current_pos = transformed_end;
                                last_control_point = Some(transformed_control);
                            },
                            PathSegment::ClosePath { .. } => {
                                if let Some(first_record) = current_shape.records.first() {
                                    if let ShapeRecord::StyleChange(style_change) = first_record {
                                        if let Some(move_to) = style_change.move_to {
                                            let first_point = Point::new(move_to.x as f64, move_to.y as f64);
                                            current_shape.records.push(ShapeRecord::Edge(
                                                shape_records::Edge {
                                                    delta: point_to_vec2d(current_pos, first_point),
                                                    control_delta: None,
                                                },
                                            ));
                                            current_pos = first_point;
                                        }
                                    }
                                }
                                last_control_point = None;
                            },
                            PathSegment::EllipticalArc { abs, rx, ry, x_axis_rotation, large_arc, sweep, x, y } => {
                                let end_point = if abs {
                                    Point::new(x as f64, y as f64)
                                } else {
                                    Point::new(current_pos.x + x as f64, current_pos.y + y as f64)
                                };

                                if rx == 0.0 || ry == 0.0 {
                                    let transformed_end = path_transform
                                        .as_ref()
                                        .map(|t| apply_transform(end_point, t))
                                        .unwrap_or(end_point);
                                    let transformed_end = group_transform
                                        .as_ref()
                                        .map(|t| apply_transform(transformed_end, t))
                                        .unwrap_or(transformed_end);

                                    current_shape.records.push(ShapeRecord::Edge(
                                        shape_records::Edge {
                                            delta: point_to_vec2d(current_pos, transformed_end),
                                            control_delta: None,
                                        },
                                    ));
                                    current_pos = transformed_end;
                                    continue;
                                }

                                let rx = rx.abs();
                                let ry = ry.abs();
                                let x_axis_rotation = x_axis_rotation.to_radians();

                                let dx = (current_pos.x - end_point.x) / 2.0;
                                let dy = (current_pos.y - end_point.y) / 2.0;

                                let cos_phi = x_axis_rotation.cos();
                                let sin_phi = x_axis_rotation.sin();

                                let x1 = cos_phi * dx + sin_phi * dy;
                                let y1 = -sin_phi * dx + cos_phi * dy;

                                let lambda = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
                                let (rx, ry) = if lambda > 1.0 {
                                    let sqrt_lambda = lambda.sqrt();
                                    (rx * sqrt_lambda, ry * sqrt_lambda)
                                } else {
                                    (rx, ry)
                                };

                                let sign = if large_arc == sweep { -1.0 } else { 1.0 };
                                let sq = ((rx * rx * ry * ry) - (rx * rx * y1 * y1) - (ry * ry * x1 * x1)) /
                                        ((rx * rx * y1 * y1) + (ry * ry * x1 * x1));
                                let sq = if sq < 0.0 { 0.0 } else { sq };
                                let coef = sign * sq.sqrt();

                                let cx1 = coef * ((rx * y1) / ry);
                                let cy1 = coef * -((ry * x1) / rx);

                                let cx = cos_phi * cx1 - sin_phi * cy1 + (current_pos.x + end_point.x) / 2.0;
                                let cy = sin_phi * cx1 + cos_phi * cy1 + (current_pos.y + end_point.y) / 2.0;

                                let start_angle = ((y1 - cy1) / ry).atan2((x1 - cx1) / rx);
                                let mut delta_angle = (((-y1 - cy1) / ry).atan2((-x1 - cx1) / rx) - start_angle) % (2.0 * std::f64::consts::PI);

                                if !sweep && delta_angle > 0.0 {
                                    delta_angle -= 2.0 * std::f64::consts::PI;
                                } else if sweep && delta_angle < 0.0 {
                                    delta_angle += 2.0 * std::f64::consts::PI;
                                }

                                let n_curves = (delta_angle.abs() * 2.0 / std::f64::consts::PI).ceil() as i32;
                                let delta_angle = delta_angle / n_curves as f64;

                                for i in 0..n_curves {
                                    let angle = start_angle + i as f64 * delta_angle;
                                    let next_angle = angle + delta_angle;

                                    let alpha = delta_angle.sin() * (delta_angle.cos() - 1.0) / delta_angle.cos();

                                    let p0 = Point::new(
                                        cx + (angle.cos() * rx * cos_phi - angle.sin() * ry * sin_phi),
                                        cy + (angle.cos() * rx * sin_phi + angle.sin() * ry * cos_phi)
                                    );
                                    let p3 = Point::new(
                                        cx + (next_angle.cos() * rx * cos_phi - next_angle.sin() * ry * sin_phi),
                                        cy + (next_angle.cos() * rx * sin_phi + next_angle.sin() * ry * cos_phi)
                                    );

                                    let control = Point::new(
                                        p0.x + alpha * (-angle.sin() * rx * cos_phi - angle.cos() * ry * sin_phi),
                                        p0.y + alpha * (-angle.sin() * rx * sin_phi + angle.cos() * ry * cos_phi)
                                    );

                                    let transformed_control = path_transform
                                        .as_ref()
                                        .map(|t| apply_transform(control, t))
                                        .unwrap_or(control);
                                    let transformed_control = group_transform
                                        .as_ref()
                                        .map(|t| apply_transform(transformed_control, t))
                                        .unwrap_or(transformed_control);

                                    let transformed_p3 = path_transform
                                        .as_ref()
                                        .map(|t| apply_transform(p3, t))
                                        .unwrap_or(p3);
                                    let transformed_p3 = group_transform
                                        .as_ref()
                                        .map(|t| apply_transform(transformed_p3, t))
                                        .unwrap_or(transformed_p3);

                                    current_shape.records.push(ShapeRecord::Edge(
                                        shape_records::Edge {
                                            delta: point_to_vec2d(current_pos, transformed_p3),
                                            control_delta: Some(point_to_vec2d(current_pos, transformed_control)),
                                        },
                                    ));

                                    current_pos = transformed_p3;
                                }
                            },
                        }
                    }

                    println!("Path processed: {} straight edges, {} curves", path_count, path_count);
                }
            }
            _ => {}
        }
    }

    // Add the final shape to the collection
    if !current_shape.records.is_empty() {
        let record_count = current_shape.records.len();
        shapes.push(current_shape);
        println!("Final shape added to collection with {} records", record_count);
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
                println!("Original shape records: {}", tag.shape.records.len());
                println!("Original fill styles: {}", tag.shape.initial_styles.fill.len());

                // Find a matching shape from the new shapes
                if let Some(new_shape) = new_shapes.first() {
                    println!("New shape records: {}", new_shape.records.len());
                    println!("New fill styles: {}", new_shape.initial_styles.fill.len());

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

    const PADDING: i32 = 200;  // 10 pixels * 20 twips/pixel
    Ok(Rect {
        x_min: min_x - PADDING,
        x_max: max_x + PADDING,
        y_min: min_y - PADDING,
        y_max: max_y + PADDING,
    })
}

fn find_next_available_id(movie: &Movie) -> u16 {
    let mut max_id = 0;
    for tag in &movie.tags {
        match tag {
            Tag::DefineShape(shape) => max_id = max_id.max(shape.id),
            Tag::DefineSprite(sprite) => max_id = max_id.max(sprite.id),
            Tag::DefineText(text) => max_id = max_id.max(text.id),
            Tag::DefineDynamicText(text) => max_id = max_id.max(text.id),
            Tag::DefineButton(button) => max_id = max_id.max(button.id),
            Tag::DefineMorphShape(shape) => max_id = max_id.max(shape.id),
            Tag::DefineBitmap(bitmap) => max_id = max_id.max(bitmap.id),
            _ => {}
        }
    }
    max_id + 1
}

fn add_new_shapes(movie: &mut Movie, shapes: &[NewShape], config_path: &Path) -> Result<(), String> {
    println!("Adding new shapes to movie...");

    for shape in shapes {
        // Resolve the source path relative to the config file's directory
        let source_path = config_path
            .parent()
            .ok_or_else(|| "Could not determine config file directory".to_string())?
            .join(&shape.source);

        println!("Processing new shape from source: {}", source_path.display());

        // Parse the SVG source into shapes
        let parsed_shapes = parse_shape_source(&source_path)?;

        if parsed_shapes.is_empty() {
            return Err(format!("No shapes found in SVG file: {}", source_path.display()));
        }

        // Use provided ID or generate a new one
        let shape_id = shape.id.unwrap_or_else(|| find_next_available_id(movie));

        // Create the shape tag
        let shape_tag = Tag::DefineShape(tags::DefineShape {
            id: shape_id,
            bounds: if let Some(bounds) = &shape.bounds {
                Rect {
                    x_min: bounds.x.min,
                    x_max: bounds.x.max,
                    y_min: bounds.y.min,
                    y_max: bounds.y.max,
                }
            } else {
                calculate_shape_bounds(&parsed_shapes[0])?
            },
            edge_bounds: None,
            has_fill_winding: false,
            has_non_scaling_strokes: false,
            has_scaling_strokes: false,
            shape: parsed_shapes[0].clone(),
        });

        // Add the new shape tag to the movie
        movie.tags.push(shape_tag);
        println!("Added new shape with ID: {}", shape_id);
    }

    Ok(())
}

fn add_new_sprites(movie: &mut Movie, sprites: &[NewSprite]) -> Result<(), String> {
    println!("Adding new sprites to movie...");

    for sprite in sprites {
        // Use provided ID or generate a new one
        let sprite_id = sprite.id.unwrap_or_else(|| find_next_available_id(movie));

        // Create the sprite tag
        let sprite_tag = Tag::DefineSprite(swf_types::tags::DefineSprite {
            id: sprite_id,
            frame_count: sprite.frame_count as usize,
            tags: sprite.tags.clone(),
        });

        // Add the new sprite tag to the movie
        movie.tags.push(sprite_tag);
        println!("Added new sprite with ID: {}", sprite_id);
    }

    Ok(())
}

fn add_new_texts(movie: &mut Movie, texts: &[NewText]) -> Result<(), String> {
    println!("Adding new text elements to movie...");

    for text in texts {
        // Use provided ID or generate a new one
        let text_id = text.id.unwrap_or_else(|| find_next_available_id(movie));

        // Create text bounds
        let bounds = Rect {
            x_min: text.bounds.x.min,
            x_max: text.bounds.x.max,
            y_min: text.bounds.y.min,
            y_max: text.bounds.y.max,
        };

        // Create a dynamic text tag
        let text_tag = Tag::DefineDynamicText(swf_types::tags::DefineDynamicText {
            id: text_id,
            bounds,
            word_wrap: text.word_wrap,
            multiline: text.multiline,
            password: false,
            readonly: text.readonly,
            auto_size: false,
            no_select: text.no_select,
            border: false,
            was_static: false,
            html: text.html,
            use_glyph_font: text.use_outlines,
            font_id: None,
            font_class: Some(text.font_class.clone()),
            font_size: Some(text.font_size),
            color: text.color,
            max_length: None,
            align: text.align,
            margin_left: text.margin_left,
            margin_right: text.margin_right,
            indent: text.indent,
            leading: text.leading,
            variable_name: None,
            text: Some(text.text.clone()),
        });

        // Add the new text tag to the movie
        movie.tags.push(text_tag);
        println!("Added new text with ID: {}", text_id);
    }

    Ok(())
}

fn add_new_elements(movie: &mut Movie, elements: &NewElements) -> Result<(), String> {
    if let Some(bitmaps) = &elements.bitmaps {
        for bitmap in bitmaps {
            let bitmap_id = bitmap.id.unwrap_or_else(|| find_next_available_id(movie));
            let bitmap_tag = Tag::DefineBitmap(swf_types::tags::DefineBitmap {
                id: bitmap_id,
                width: bitmap.width,
                height: bitmap.height,
                media_type: swf_types::ImageType::Png,  // Use the re-exported ImageType
                data: bitmap.data.clone(),
            });
            movie.tags.push(bitmap_tag);
        }
    }

    if let Some(buttons) = &elements.buttons {
        for button in buttons {
            for state in &button.states {
                let button_tag = Tag::DefineButton(state.clone());
                movie.tags.push(button_tag);
            }
        }
    }

    if let Some(scenes) = &elements.scenes {
        // Find or create the scene data tag
        let mut scene_data_tag = None;
        for tag in &mut movie.tags {
            if let Tag::DefineSceneAndFrameLabelData(data) = tag {
                scene_data_tag = Some(data);
                break;
            }
        }

        if let Some(data) = scene_data_tag {
            for scene in scenes {
                data.scenes.push(swf_types::Scene {
                    name: scene.name.clone(),
                    offset: scene.offset,
                });
            }
        } else {
            // Create new scene data tag if none exists
            let new_scenes: Vec<_> = scenes.iter()
                .map(|s| swf_types::Scene {
                    name: s.name.clone(),
                    offset: s.offset,
                })
                .collect();

            movie.tags.push(Tag::DefineSceneAndFrameLabelData(
                swf_types::tags::DefineSceneAndFrameLabelData {
                    scenes: new_scenes,
                    labels: Vec::new(),
                }
            ));
        }
    }

    Ok(())
}

fn remove_swf_elements(movie: &mut Movie, elements: &RemoveElements) -> Result<(), String> {
    println!("Starting element removal process...");

    // Create a set of IDs to remove for each type
    let shape_ids: std::collections::HashSet<_> = elements.shapes.as_ref().map(|v| v.iter().copied().collect()).unwrap_or_default();
    let sprite_ids: std::collections::HashSet<_> = elements.sprites.as_ref().map(|v| v.iter().copied().collect()).unwrap_or_default();
    let text_ids: std::collections::HashSet<_> = elements.texts.as_ref().map(|v| v.iter().copied().collect()).unwrap_or_default();
    let button_ids: std::collections::HashSet<_> = elements.buttons.as_ref().map(|v| v.iter().copied().collect()).unwrap_or_default();
    let bitmap_ids: std::collections::HashSet<_> = elements.bitmaps.as_ref().map(|v| v.iter().copied().collect()).unwrap_or_default();
    let frame_labels: std::collections::HashSet<_> = elements.frames.as_ref().map(|v| v.iter().cloned().collect()).unwrap_or_default();
    let scene_names: std::collections::HashSet<_> = elements.scenes.as_ref().map(|v| v.iter().cloned().collect()).unwrap_or_default();

    // First pass: Remove all references to the elements in the main timeline
    movie.tags.retain(|tag| {
        match tag {
            Tag::PlaceObject(place) => {
                let char_id = place.character_id.unwrap_or(0);
                !shape_ids.contains(&char_id) &&
                !sprite_ids.contains(&char_id) &&
                !text_ids.contains(&char_id) &&
                !button_ids.contains(&char_id) &&
                !bitmap_ids.contains(&char_id)
            },
            Tag::FrameLabel(label) => !frame_labels.contains(&label.name),
            _ => true
        }
    });

    // Second pass: Remove the actual element definitions
    movie.tags.retain(|tag| {
        match tag {
            Tag::DefineShape(shape) => !shape_ids.contains(&shape.id),
            Tag::DefineSprite(sprite) => !sprite_ids.contains(&sprite.id),
            Tag::DefineText(text) => !text_ids.contains(&text.id),
            Tag::DefineDynamicText(text) => !text_ids.contains(&text.id),
            Tag::DefineButton(button) => !button_ids.contains(&button.id),
            Tag::DefineBitmap(bitmap) => !bitmap_ids.contains(&bitmap.id),
            Tag::DefineSceneAndFrameLabelData(data) => {
                // For scene data, we'll keep the tag but filter its contents if needed
                if scene_names.is_empty() {
                    true
                } else {
                    // Create a filtered copy of scenes
                    let filtered_scenes = data.scenes
                        .iter()
                        .filter(|scene| !scene_names.contains(&scene.name))
                        .cloned()
                        .collect::<Vec<_>>();

                    // Only keep if we have scenes or labels
                    !filtered_scenes.is_empty() || !data.labels.is_empty()
                }
            },
            _ => true
        }
    });

    // Update scene data with filtered scenes
    if !scene_names.is_empty() {
        for tag in &mut movie.tags {
            if let Tag::DefineSceneAndFrameLabelData(data) = tag {
                data.scenes = data.scenes
                    .iter()
                    .filter(|scene| !scene_names.contains(&scene.name))
                    .cloned()
                    .collect();
            }
        }
    }

    // Clean up any references to removed elements in sprite tags
    for tag in &mut movie.tags {
        if let Tag::DefineSprite(sprite) = tag {
            sprite.tags.retain(|tag| {
                match tag {
                    Tag::PlaceObject(place) => {
                        let char_id = place.character_id.unwrap_or(0);
                        !shape_ids.contains(&char_id) &&
                        !sprite_ids.contains(&char_id) &&
                        !text_ids.contains(&char_id) &&
                        !button_ids.contains(&char_id) &&
                        !bitmap_ids.contains(&char_id)
                    },
                    Tag::FrameLabel(label) => !frame_labels.contains(&label.name),
                    _ => true
                }
            });
        }
    }

    println!("Element removal completed successfully");
    Ok(())
}

fn apply_modifications(movie: &mut Movie, config: &SwfModification, config_path: &Path) -> Result<(), String> {
    if let Some(bounds) = &config.bounds {
        movie.header.frame_size.x_min = bounds.x.min;
        movie.header.frame_size.x_max = bounds.x.max;
        movie.header.frame_size.y_min = bounds.y.min;
        movie.header.frame_size.y_max = bounds.y.max;
    }

    // Apply existing tag modifications
    for modification in &config.modifications {
        apply_tag_modification(movie, modification)?;
    }

    // Handle new elements if present
    if let Some(new_elements) = &config.new_elements {
        if let Some(shapes) = &new_elements.shapes {
            add_new_shapes(movie, shapes, config_path)?;
        }
        if let Some(sprites) = &new_elements.sprites {
            add_new_sprites(movie, sprites)?;
        }
        if let Some(texts) = &new_elements.texts {
            add_new_texts(movie, texts)?;
        }
    }

    // Handle element removal if present
    if let Some(remove_elements) = &config.remove_elements {
        remove_swf_elements(movie, remove_elements)?;
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
            (Tag::DefineDynamicText(tag), "DefineDynamicTextTag") if tag.id == modification.id => {
                if let Some(text) = modification.properties.get("text") {
                    tag.text = serde_json::from_value(text.clone())
                        .map_err(|e| format!("Failed to parse dynamic text: {}", e))?;
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

            (Tag::DoAbc(tag), "DoAbcTag") if modification.tag == "DoAbcTag" => {
                if let Some(data) = modification.properties.get("data") {
                    tag.data = serde_json::from_value(data.clone())
                        .map_err(|e| format!("Failed to parse ABC data: {}", e))?;
                }
            }
            (Tag::DoAction(tag), "DoActionTag") if modification.tag == "DoActionTag" => {
                if let Some(actions) = modification.properties.get("actions") {
                    tag.actions = serde_json::from_value(actions.clone())
                        .map_err(|e| format!("Failed to parse actions: {}", e))?;
                }
            }
            (Tag::FileAttributes(tag), "FileAttributesTag") if modification.tag == "FileAttributesTag" => {
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

    // Handle shape transparency
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
                    let new_tag = Tag::DefineShape(tags::DefineShape {
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
                // Find the SWF path from the mappings
                let swf_path = config.swf_mappings.iter()
                    .find(|m| m.mod_name == mod_config.name)
                    .map(|m| m.swf_path.clone())
                    .ok_or_else(|| format!("No SWF mapping found for mod: {}", mod_config.name))?;

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

fn apply_actionscript_patches(movie: &mut Movie, patches: &[ActionScriptPatch], config_path: &str, handle: AppHandle) -> Result<(), String> {
    // Create a temporary directory for compilation
    let temp_dir = TempDir::new()
        .map_err(|e| format!("Failed to create temporary directory: {}", e))?;

    // Get the config file's directory
    let config_dir = Path::new(config_path)
        .parent()
        .ok_or_else(|| "Could not determine config file directory".to_string())?;

    for patch in patches {
        // Read the ActionScript source file
        let source_path = config_dir.join(&patch.source_file);
        let mut source_code = fs::read_to_string(&source_path)
            .map_err(|e| format!("Failed to read ActionScript file '{}': {}", patch.source_file, e))?;

        // If package_name is provided, ensure the code has the correct package declaration
        if let Some(package_name) = &patch.package_name {
            // Check if there's an existing package declaration
            if source_code.contains("package ") {
                // Replace existing package declaration
                source_code = source_code.replace(
                    &extract_package_declaration(&source_code)
                        .unwrap_or_else(|| "package ".to_string()),
                    &format!("package {} ", package_name)
                );
            } else {
                // Add package declaration at the start
                source_code = format!("package {} {{\n{}\n}}", package_name, source_code);
            }
        }

        // If class_name is provided and we're in Replace mode, ensure the class has the correct name
        if patch.insert_mode == ActionScriptInsertMode::Replace {
            if let Some(class_name) = &patch.class_name {
                // Check if there's an existing class declaration
                if source_code.contains("class ") {
                    // Replace existing class name
                    source_code = source_code.replace(
                        &extract_class_declaration(&source_code)
                            .unwrap_or_else(|| "class ".to_string()),
                        &format!("class {} ", class_name)
                    );
                }
            }
        }

        // Create a temporary SWF for compilation
        let temp_swf_path = temp_dir.path().join("temp.swf");

        // Write the current movie to the temp SWF
        let swf_data = emit_swf(&movie, swf_types::CompressionMethod::None)
            .map_err(|e| format!("Failed to write temporary SWF: {}", e))?;
        fs::write(&temp_swf_path, swf_data)
            .map_err(|e| format!("Failed to write temporary SWF: {}", e))?;

        // Write the modified ActionScript file to the temp directory
        let temp_as_path = temp_dir.path().join("Main.as");
        fs::write(&temp_as_path, source_code)
            .map_err(|e| format!("Failed to write temporary AS file: {}", e))?;

        // Compile the ActionScript using JPEXS
        let abc_data = compile_with_jpexs(handle.clone(), &temp_as_path, &temp_swf_path)?;

        // Create a new DoABC tag with the compiled code
        let new_tag = Tag::DoAbc(swf_types::tags::DoAbc {
            header: None,
            data: abc_data,
        });

        // Add or replace the tag based on insert mode
        match patch.insert_mode {
            ActionScriptInsertMode::Add => {
                movie.tags.push(new_tag);
            },
            ActionScriptInsertMode::Replace => {
                if let Some(class_name) = &patch.class_name {
                    // Try to find and replace the existing ABC tag with matching class name
                    let mut found = false;
                    for tag in &mut movie.tags {
                        if let Tag::DoAbc(abc_tag) = tag {
                            if contains_class_name(&abc_tag.data, class_name) {
                                *tag = new_tag.clone();
                                found = true;
                                break;
                            }
                        }
                    }
                    if !found {
                        movie.tags.push(new_tag);
                    }
                } else {
                    // If no class name specified, replace first DoAbc tag
                    let mut found = false;
                    for tag in &mut movie.tags {
                        if let Tag::DoAbc(_) = tag {
                            *tag = new_tag.clone();
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        movie.tags.push(new_tag);
                    }
                }
            },
        }

        // Handle symbol class bindings if present
        if let Some(bindings) = &patch.symbol_bindings {
            // Find or create a SymbolClass tag
            let mut symbol_class_tag = None;
            for tag in &mut movie.tags {
                if let Tag::SymbolClass(symbol_tag) = tag {
                    symbol_class_tag = Some(symbol_tag);
                    break;
                }
            }

            // Create new symbol bindings
            let mut new_symbols = Vec::new();
            if let Some(symbol_tag) = symbol_class_tag {
                // Keep existing bindings that aren't being replaced
                for binding in &symbol_tag.symbols {
                    if !bindings.iter().any(|b| b.symbol_id == binding.id) {
                        new_symbols.push(binding.clone());
                    }
                }
            }

            // Add new bindings
            for binding in bindings {
                new_symbols.push(swf_types::NamedId {
                    id: binding.symbol_id,
                    name: binding.class_name.clone(),
                });
            }

            // Create or update the SymbolClass tag
            let symbol_class_tag = Tag::SymbolClass(swf_types::tags::SymbolClass {
                symbols: new_symbols,
            });

            // Replace existing tag or add new one
            let mut found = false;
            for tag in &mut movie.tags {
                if let Tag::SymbolClass(_) = tag {
                    *tag = symbol_class_tag.clone();
                    found = true;
                    break;
                }
            }
            if !found {
                movie.tags.push(symbol_class_tag);
            }
        }
    }

    Ok(())
}

// Helper function to extract package declaration from ActionScript code
fn extract_package_declaration(source: &str) -> Option<String> {
    if let Some(start) = source.find("package ") {
        if let Some(end) = source[start..].find("{") {
            return Some(source[start..start + end].trim().to_string());
        }
    }
    None
}

// Helper function to extract class declaration from ActionScript code
fn extract_class_declaration(source: &str) -> Option<String> {
    if let Some(start) = source.find("class ") {
        if let Some(end) = source[start..].find("{") {
            let class_decl = source[start..start + end].trim();
            if let Some(extends_idx) = class_decl.find("extends") {
                return Some(class_decl[..extends_idx].trim().to_string());
            }
            if let Some(implements_idx) = class_decl.find("implements") {
                return Some(class_decl[..implements_idx].trim().to_string());
            }
            return Some(class_decl.to_string());
        }
    }
    None
}

// Helper function to check if ABC data contains a class name
fn contains_class_name(abc_data: &[u8], class_name: &str) -> bool {
    // Simple string search in the ABC data
    // This is a basic implementation - in the future, we could properly parse the ABC format
    let class_bytes = class_name.as_bytes();
    abc_data.windows(class_bytes.len()).any(|window| window == class_bytes)
}

fn check_java_installation() -> Result<(), String> {
    let output = Command::new("java")
        .arg("-version")
        .output()
        .map_err(|_| "Java is not installed or not accessible".to_string())?;

    if !output.status.success() {
        return Err("Failed to verify Java installation".to_string());
    }
    Ok(())
}

fn compile_with_jpexs(handle: AppHandle, as_path: &Path, swf_path: &Path) -> Result<Vec<u8>, String> {
    // Check Java installation first
    check_java_installation()?;

    // Create a temporary directory for JPEXS output
    let output_dir = TempDir::new()
        .map_err(|e| format!("Failed to create temporary output directory: {}", e))?;

    // Get the path to ffdec.jar from the bundled resources
    let resource_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?
        .parent()
        .ok_or_else(|| "Failed to get parent directory".to_string())?
        .join("resources")
        .join("ffdec.jar");

    println!("Using JPEXS from: {}", resource_path.display());

    // Create a temporary output SWF path
    let output_swf = output_dir.path().join("output.swf");

    // Run JPEXS to import the ActionScript
    let status = Command::new("java")
        .args([
            "-jar",
            resource_path.to_str().unwrap(),
            "-importScript",
            as_path.to_str().unwrap(),
            swf_path.to_str().unwrap(),
            output_swf.to_str().unwrap(),
        ])
        .status()
        .map_err(|e| format!("Failed to execute JPEXS: {}", e))?;

    if !status.success() {
        return Err("JPEXS script import failed".to_string());
    }

    // Now we need to extract the ABC tag from the output SWF
    // First convert the SWF to JSON so we can find the ABC tag
    let temp_json = output_dir.path().join("temp.json");
    convert_swf_to_json(
        handle,
        output_swf.to_string_lossy().to_string(),
        temp_json.to_string_lossy().to_string(),
    )?;

    // Read and parse the JSON
    let json_data = fs::read_to_string(&temp_json)
        .map_err(|e| format!("Failed to read temporary JSON: {}", e))?;
    let movie: Movie = serde_json::from_str(&json_data)
        .map_err(|e| format!("Failed to parse temporary JSON: {}", e))?;

    // Find the first DoAbc tag and return its data
    for tag in movie.tags {
        if let Tag::DoAbc(abc_tag) = tag {
            return Ok(abc_tag.data);
        }
    }

    Err("No ABC tag found in compiled SWF".to_string())
}
