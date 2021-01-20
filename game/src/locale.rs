pub fn key_bindings() -> std::collections::HashMap<&'static str, game_lib::Event> {
    let mut ret = std::collections::HashMap::new();
    ret.insert("h", game_lib::Event::Left);
    ret.insert("j", game_lib::Event::Down);
    ret.insert("k", game_lib::Event::Up);
    ret.insert("l", game_lib::Event::Right);

    ret.insert("w", game_lib::Event::Up);
    ret.insert("a", game_lib::Event::Left);
    ret.insert("s", game_lib::Event::Down);
    ret.insert("d", game_lib::Event::Right);

    ret.insert("ArrowUp", game_lib::Event::Up);
    ret.insert("ArrowDown", game_lib::Event::Down);
    ret.insert("ArrowLeft", game_lib::Event::Left);
    ret.insert("ArrowRight", game_lib::Event::Right);
    ret
}
