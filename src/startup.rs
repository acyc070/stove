use super::*;

pub fn set_icon(windows: NonSend<bevy::winit::WinitWindows>) {
    let icon =
        winit::window::Icon::from_rgba(include_bytes!("../assets/pot.rgba").to_vec(), 64, 64)
            .unwrap();
    for window in windows.windows.values() {
        window.set_window_icon(Some(icon.clone()))
    }
}

pub fn check_updates(mut command: EventWriter<Command>) {
    use update_informer::Check;
    if let Ok(Some(new)) = update_informer::new(
        update_informer::registry::GitHub,
        "bananaturtlesandwich/stove",
        env!("CARGO_PKG_VERSION"),
    )
    .check_version()
    {
        command.send(Command::Notif {
            // yes i'm petty and hate the v prefix
            message: format!(
                "{}.{}.{} now available!",
                new.semver().major,
                new.semver().minor,
                new.semver().patch
            ),
            kind: Info,
        })
    }
}

pub fn check_args(mut commands: EventWriter<Command>) {
    let Some(path) = std::env::args().nth(1) else {
        return;
    };
    let path = std::path::PathBuf::from(path);
    if !path.exists() {
        commands.send(Command::Notif {
            message: "the given path does not exist".into(),
            kind: Error,
        });
        return;
    }
    commands.send(Command::Open(path))
}
