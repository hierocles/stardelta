use std::process::Command;
use tauri::command;

#[command]
pub fn convert_swf_to_xml(swf_path: String, xml_path: String) -> Result<(), String> {
    log::trace!("Converting SWF to XML: {} -> {}", swf_path, xml_path);

    let output = Command::new("java")
        .arg("-jar")
        .arg("ffdec.jar")
        .arg("swf2xml")
        .arg(&swf_path)
        .arg(&xml_path)
        .output()
        .map_err(|e| {
            log::error!("Failed to execute command: {}", e);
            e.to_string()
        })?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        log::error!("Command failed: {}", error);
        return Err(format!("Conversion failed: {}", error));
    }

    Ok(())
}
