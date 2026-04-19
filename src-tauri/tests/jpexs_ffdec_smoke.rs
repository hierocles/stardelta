//! Manual smoke check for Java + FFDec (JPEXS) when ActionScript patches are used.
//!
//! Run from `src-tauri`:
//! `STARDELTA_RUN_JPEXS_INTEGRATION=1 FFDEC_JAR=/path/to/ffdec.jar cargo test --test jpexs_ffdec_smoke -- --ignored`

use std::path::Path;
use std::process::Command;

#[test]
#[ignore = "set STARDELTA_RUN_JPEXS_INTEGRATION=1 and FFDEC_JAR (or STARDELTA_FFDEC_JAR)"]
fn java_and_ffdec_jar_run() {
    assert_eq!(
        std::env::var("STARDELTA_RUN_JPEXS_INTEGRATION")
            .as_deref()
            .unwrap_or(""),
        "1",
        "set STARDELTA_RUN_JPEXS_INTEGRATION=1 to enable this test"
    );

    let jar = std::env::var("FFDEC_JAR")
        .or_else(|_| std::env::var("STARDELTA_FFDEC_JAR"))
        .expect("FFDEC_JAR or STARDELTA_FFDEC_JAR must be set");
    let jar_path = Path::new(&jar);
    assert!(
        jar_path.is_file(),
        "ffdec.jar not found at {}",
        jar_path.display()
    );

    let java_ok = Command::new("java")
        .arg("-version")
        .status()
        .expect("failed to spawn java -version");
    assert!(java_ok.success(), "java -version failed");
}
