use wit_bindgen::generate;


generate!({
    path: "../module.wit",
    world: "enum-flags",
});

struct Component;

impl Guest for Component {
    fn get_mode() -> Mode {
        Mode::ReadOnly
    }

    fn toggle_mode(mode: Mode) -> Mode {
        match mode {
            Mode::Off => Mode::Turbo,
            Mode::Turbo => Mode::ReadOnly,
            Mode::ReadOnly => Mode::Off,
        }
    }

    fn get_perms() -> Perms {
        Perms::READ | Perms::WRITE
    }

    fn echo_perms(perms: Perms) -> Perms {
        perms
    }
}

export!(Component);
