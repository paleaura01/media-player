// tests/integration_test.rs
use media_player::Player;

#[test]
fn test_play_nonexistent_file() {
    let mut player = Player::new();
    // Try to play a file that doesn't exist, should return an error.
    let result = player.play("nonexistent_file.mp3");
    assert!(result.is_err());
}
