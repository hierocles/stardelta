//! JSON patch merge for `swf_types::Tag` variants (modification file `tag` + `properties`).

use serde::{Deserialize, Serialize};
use serde_json;
use swf_types::{tags, Movie, SRgb8, StraightSRgba8, Tag};

#[derive(Debug, Deserialize)]
pub(crate) struct TagModification {
    pub tag: String,
    #[serde(default)]
    pub id: u16,
    pub properties: serde_json::Value,
    #[serde(default)]
    pub tag_index: Option<usize>,
    #[serde(default)]
    pub depth: Option<u16>,
    #[serde(default)]
    pub character_id: Option<u16>,
    #[serde(default)]
    pub frame_label_name: Option<String>,
}

fn merge_json_object_into(base: &mut serde_json::Value, overlay: &serde_json::Value) {
    match (base, overlay) {
        (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
            for (k, v) in b {
                if let Some(existing) = a.get_mut(k) {
                    if existing.is_object() && v.is_object() {
                        merge_json_object_into(existing, v);
                    } else {
                        *existing = v.clone();
                    }
                } else {
                    a.insert(k.clone(), v.clone());
                }
            }
        }
        (a, b) => *a = b.clone(),
    }
}

fn merge_json_value_into<T>(target: &mut T, overlay: &serde_json::Value) -> Result<(), String>
where
    T: Serialize + serde::de::DeserializeOwned,
{
    let mut base = serde_json::to_value(&*target).map_err(|e| format!("{}", e))?;
    merge_json_object_into(&mut base, overlay);
    *target = serde_json::from_value(base).map_err(|e| format!("{}", e))?;
    Ok(())
}

fn normalize_file_attributes_properties(props: &serde_json::Value) -> serde_json::Value {
    let Some(obj) = props.as_object() else {
        return props.clone();
    };
    let mut o = obj.clone();
    if let Some(v) = o.remove("actionScript3") {
        o.insert("use_as3".to_string(), v);
    }
    if let Some(v) = o.remove("hasMetadata") {
        o.insert("has_metadata".to_string(), v);
    }
    if let Some(v) = o.remove("useNetwork") {
        o.insert("use_network".to_string(), v);
    }
    if let Some(v) = o.remove("useGPU") {
        o.insert("use_gpu".to_string(), v);
    }
    serde_json::Value::Object(o)
}

fn normalize_place_object_properties(props: &serde_json::Value) -> serde_json::Value {
    let Some(obj) = props.as_object() else {
        return props.clone();
    };
    let mut o = obj.clone();
    if let Some(v) = o.remove("colorTransform") {
        o.insert("color_transform".to_string(), v);
    }
    serde_json::Value::Object(o)
}

fn place_object_matches_filter(place: &tags::PlaceObject, m: &TagModification) -> bool {
    if let Some(d) = m.depth {
        if place.depth != d {
            return false;
        }
    }
    if let Some(cid) = m.character_id {
        if place.character_id != Some(cid) {
            return false;
        }
    }
    true
}

fn remove_object_matches_filter(ro: &tags::RemoveObject, m: &TagModification) -> bool {
    if let Some(d) = m.depth {
        if ro.depth != d {
            return false;
        }
    }
    if let Some(cid) = m.character_id {
        if ro.character_id != Some(cid) {
            return false;
        }
    }
    true
}

fn frame_label_matches_filter(label: &tags::FrameLabel, m: &TagModification) -> bool {
    if let Some(ref n) = m.frame_label_name {
        return label.name == *n;
    }
    true
}

fn id_match(tag_id: u16, modification: &TagModification, indexed: bool) -> bool {
    indexed || tag_id == modification.id
}

fn try_merge_one_tag(
    tag: &mut Tag,
    modification: &TagModification,
    indexed: bool,
) -> Result<bool, String> {
    let tn = modification.tag.as_str();

    match tn {
        "CsmTextSettingsTag" => {
            let Tag::CsmTextSettings(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.text_id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DefineBinaryDataTag" => {
            let Tag::DefineBinaryData(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DefineBitmapTag" => {
            let Tag::DefineBitmap(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DefineButtonTag" => {
            let Tag::DefineButton(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DefineButtonColorTransformTag" => {
            let Tag::DefineButtonColorTransform(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.button_id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DefineButtonSoundTag" => {
            let Tag::DefineButtonSound(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.button_id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DefineCffFontTag" => {
            let Tag::DefineCffFont(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DefineDynamicTextTag" => {
            let Tag::DefineDynamicText(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DefineFontTag" => {
            let Tag::DefineFont(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DefineFontAlignZonesTag" => {
            let Tag::DefineFontAlignZones(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.font_id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DefineFontInfoTag" => {
            let Tag::DefineFontInfo(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.font_id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DefineFontNameTag" => {
            let Tag::DefineFontName(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.font_id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DefineGlyphFontTag" => {
            let Tag::DefineGlyphFont(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DefineJpegTablesTag" => {
            let Tag::DefineJpegTables(t) = tag else {
                return Ok(false);
            };
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DefineMorphShapeTag" => {
            let Tag::DefineMorphShape(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DefineScalingGridTag" => {
            let Tag::DefineScalingGrid(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.character_id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DefineSceneAndFrameLabelDataTag" => {
            let Tag::DefineSceneAndFrameLabelData(t) = tag else {
                return Ok(false);
            };
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DefineShapeTag" => {
            let Tag::DefineShape(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.id, modification, indexed) {
                return Ok(false);
            }
            // Full-object merge when `shape` is provided (matches exported JSON).
            if modification.properties.get("shape").is_some() {
                merge_json_value_into(t, &modification.properties)?;
                return Ok(true);
            }
            // Legacy StarDelta shortcuts (properties at top level, not under `shape`).
            if let Some(v) = modification.properties.get("bounds") {
                t.bounds = serde_json::from_value(v.clone())
                    .map_err(|e| format!("Failed to parse shape bounds: {}", e))?;
            }
            if let Some(v) = modification.properties.get("records") {
                t.shape.records = serde_json::from_value(v.clone())
                    .map_err(|e| format!("Failed to parse shape records: {}", e))?;
            }
            if let Some(v) = modification.properties.get("styles") {
                t.shape.initial_styles = serde_json::from_value(v.clone())
                    .map_err(|e| format!("Failed to parse shape styles: {}", e))?;
            } else {
                if let Some(v) = modification.properties.get("fillStyles") {
                    t.shape.initial_styles.fill = serde_json::from_value(v.clone())
                        .map_err(|e| format!("Failed to parse fill styles: {}", e))?;
                }
                if let Some(v) = modification.properties.get("lineStyles") {
                    t.shape.initial_styles.line = serde_json::from_value(v.clone())
                        .map_err(|e| format!("Failed to parse line styles: {}", e))?;
                }
            }
            Ok(true)
        }
        "DefineSoundTag" => {
            let Tag::DefineSound(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DefineSpriteTag" => {
            let Tag::DefineSprite(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DefineTextTag" => {
            let Tag::DefineText(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DefineVideoStreamTag" => {
            let Tag::DefineVideoStream(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "EnablePostscriptTag" => {
            let Tag::EnablePostscript = tag else {
                return Ok(false);
            };
            Ok(true)
        }
        "DoAbcTag" => {
            let Tag::DoAbc(t) = tag else {
                return Ok(false);
            };
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DoActionTag" => {
            let Tag::DoAction(t) = tag else {
                return Ok(false);
            };
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "DoInitActionTag" => {
            let Tag::DoInitAction(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.sprite_id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "EnableDebuggerTag" => {
            let Tag::EnableDebugger(t) = tag else {
                return Ok(false);
            };
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "ExportAssetsTag" => {
            let Tag::ExportAssets(t) = tag else {
                return Ok(false);
            };
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "FileAttributesTag" => {
            let Tag::FileAttributes(t) = tag else {
                return Ok(false);
            };
            let norm = normalize_file_attributes_properties(&modification.properties);
            merge_json_value_into(t, &norm)?;
            Ok(true)
        }
        "FrameLabelTag" => {
            let Tag::FrameLabel(t) = tag else {
                return Ok(false);
            };
            if !indexed && !frame_label_matches_filter(t, modification) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "ImportAssetsTag" => {
            let Tag::ImportAssets(t) = tag else {
                return Ok(false);
            };
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "MetadataTag" => {
            let Tag::Metadata(t) = tag else {
                return Ok(false);
            };
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "PlaceObjectTag" => {
            let Tag::PlaceObject(t) = tag else {
                return Ok(false);
            };
            if !indexed && !place_object_matches_filter(t, modification) {
                return Ok(false);
            }
            let norm = normalize_place_object_properties(&modification.properties);
            merge_json_value_into(t, &norm)?;
            Ok(true)
        }
        "ProtectTag" => {
            let Tag::Protect(t) = tag else {
                return Ok(false);
            };
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "RawTag" => {
            let Tag::Raw(t) = tag else {
                return Ok(false);
            };
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "RawBodyTag" => {
            let Tag::RawBody(t) = tag else {
                return Ok(false);
            };
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "RemoveObjectTag" => {
            let Tag::RemoveObject(t) = tag else {
                return Ok(false);
            };
            if !indexed && !remove_object_matches_filter(t, modification) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "ScriptLimitsTag" => {
            let Tag::ScriptLimits(t) = tag else {
                return Ok(false);
            };
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "SetBackgroundColorTag" => {
            let Tag::SetBackgroundColor(t) = tag else {
                return Ok(false);
            };
            if let Some(c) = modification.properties.get("backgroundColor") {
                let rgba: StraightSRgba8 = serde_json::from_value(c.clone())
                    .map_err(|e| format!("Failed to parse backgroundColor: {}", e))?;
                t.color = SRgb8 {
                    r: rgba.r,
                    g: rgba.g,
                    b: rgba.b,
                };
            }
            if modification.properties.get("color").is_some() {
                merge_json_value_into(&mut t.color, modification.properties.get("color").unwrap())?;
            }
            Ok(true)
        }
        "SetTabIndexTag" => {
            let Tag::SetTabIndex(t) = tag else {
                return Ok(false);
            };
            if !indexed && t.depth != modification.id {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "ShowFrameTag" => {
            let Tag::ShowFrame = tag else {
                return Ok(false);
            };
            Ok(true)
        }
        "SoundStreamBlockTag" => {
            let Tag::SoundStreamBlock(t) = tag else {
                return Ok(false);
            };
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "SoundStreamHeadTag" => {
            let Tag::SoundStreamHead(t) = tag else {
                return Ok(false);
            };
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "StartSoundTag" => {
            let Tag::StartSound(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.sound_id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "StartSound2Tag" => {
            let Tag::StartSound2(t) = tag else {
                return Ok(false);
            };
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "SymbolClassTag" => {
            let Tag::SymbolClass(t) = tag else {
                return Ok(false);
            };
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "TelemetryTag" => {
            let Tag::Telemetry(t) = tag else {
                return Ok(false);
            };
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        "VideoFrameTag" => {
            let Tag::VideoFrame(t) = tag else {
                return Ok(false);
            };
            if !id_match(t.video_id, modification, indexed) {
                return Ok(false);
            }
            merge_json_value_into(t, &modification.properties)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub(crate) fn apply_tag_modification(
    movie: &mut Movie,
    modification: &TagModification,
) -> Result<(), String> {
    if let Some(idx) = modification.tag_index {
        if idx >= movie.tags.len() {
            return Err(format!(
                "tag_index {} is out of range (tag count {})",
                idx,
                movie.tags.len()
            ));
        }
        let applied = try_merge_one_tag(&mut movie.tags[idx], modification, true)?;
        if !applied {
            return Err(format!(
                "tag_index {}: tag at index does not match modification.tag {:?}",
                idx, modification.tag
            ));
        }
        return Ok(());
    }

    for tag in &mut movie.tags {
        let _ = try_merge_one_tag(tag, modification, false)?;
    }
    Ok(())
}
