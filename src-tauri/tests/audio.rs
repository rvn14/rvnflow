use rodio::Source;
use rvnflow_lib::services::audio::decode;

#[test]
fn original_sounds_decode_and_are_short() {
    for bytes in [
        include_bytes!("../sounds/chime.wav").as_slice(),
        include_bytes!("../sounds/bell.wav").as_slice(),
        include_bytes!("../sounds/digital.wav").as_slice(),
        include_bytes!("../sounds/mechanical.wav").as_slice(),
        include_bytes!("../sounds/soft.wav").as_slice(),
    ] {
        let sound = decode(bytes.to_vec()).unwrap();
        assert!(sound.total_duration().unwrap().as_secs_f32() < 2.0);
    }
}
#[test]
fn invalid_or_oversized_audio_is_rejected() {
    assert!(decode(vec![]).is_err());
    assert!(decode(b"not an mp3".to_vec()).is_err());
    assert!(decode(vec![0; 20 * 1024 * 1024 + 1]).is_err());
}
