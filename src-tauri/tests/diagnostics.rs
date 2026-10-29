use rvnflow_lib::infrastructure::diagnostics;
#[test]
fn diagnostic_rotation_is_bounded_and_uses_codes_only() {
    let folder = tempfile::tempdir().unwrap();
    diagnostics::init(folder.path().into());
    std::fs::write(folder.path().join("diagnostics.log"), vec![b'x'; 262_145]).unwrap();
    diagnostics::record("startup", "recovery-required");
    let current = std::fs::read_to_string(folder.path().join("diagnostics.log")).unwrap();
    assert!(current.len() < 100);
    assert!(current.contains("startup recovery-required"));
    assert!(folder.path().join("diagnostics.previous.log").exists());
}
