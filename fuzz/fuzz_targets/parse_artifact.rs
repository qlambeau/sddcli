#![no_main]

use artifact_filesystem::parse_artifact;
use domain::{ArtifactKind, ArtifactPath};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: Vec<u8>| {
    let text = String::from_utf8_lossy(&input);
    let Some(path) = ArtifactPath::try_new("specs/fuzz/scenarios.feature").ok() else {
        return;
    };
    let _ = parse_artifact(path, ArtifactKind::Gherkin, &text);
});
