use std::net::ToSocketAddrs;

use bevy::{input::{keyboard::{Key, KeyboardInput}, ButtonState}, prelude::*};
use bevy_rapier3d::render::DebugRenderContext;

use crate::{audio::Music, game::hud::HudDebugState, ui::chat::{ChatInput, ChatType, PendingMessages}, AppState, Settings};

pub enum SettingToggle {
    LobbyMode,
    Music,
    Sfx,
    Hitboxes,
    Debug,
}

pub enum SettingValue {
    LobbyMode(bool),
    LobbyUrl(String),
}

#[derive(Event)]
pub enum GameCommand {
    Toggle(SettingToggle),
    Set(SettingValue),
}

pub fn execute_cmds(
    mut commands: EventReader<GameCommand>,
    mut settings: ResMut<Settings>,
    mut pending_msgs: ResMut<PendingMessages>,
    music_sink: Query<&AudioSink, With<Music>>,
    mut debug_render: ResMut<DebugRenderContext>,
    mut debug_hud_state: ResMut<NextState<HudDebugState>>,
    app_state: Res<State<AppState>>,
) {
    for command in commands.read() {
        match command {
            GameCommand::Toggle(setting) => {
                match setting {
                    SettingToggle::LobbyMode => {
                        settings.local_lobby = !settings.local_lobby;
                        pending_msgs.0.push(format!("[INFO] local_lobby has been set to {}", settings.local_lobby));
                    }
                    SettingToggle::Music => {
                        settings.music_enabled = !settings.music_enabled;
                        for music in music_sink.iter() {
                            music.toggle();
                        }
                        pending_msgs.0.push(format!("[INFO] music_enabled has been set to {}", settings.music_enabled));
                    }
                    SettingToggle::Sfx => {
                        settings.sfx_enabled = !settings.sfx_enabled;
                        pending_msgs.0.push(format!("[INFO] sfx_enabled has been set to {}", settings.sfx_enabled));
                    }
                    SettingToggle::Hitboxes => {
                        settings.hitboxes_enabled = !settings.hitboxes_enabled;
                        debug_render.enabled = !debug_render.enabled;
                        pending_msgs.0.push(format!("[INFO] hitboxes_enabled has been set to {}", settings.hitboxes_enabled));
                    }
                    SettingToggle::Debug => {
                        settings.debug_hud_enabled = !settings.debug_hud_enabled;
                        if *app_state.get() == AppState::InGame {
                            match settings.debug_hud_enabled {
                                true => debug_hud_state.set(HudDebugState::Enabled),
                                false => debug_hud_state.set(HudDebugState::Disabled),
                            }
                        }
                        pending_msgs.0.push(format!("[INFO] debug_hud_enabled has been set to {}", settings.debug_hud_enabled));
                    }
                }
            }
            GameCommand::Set(setting) => {
                match setting {
                    SettingValue::LobbyUrl(addr) => {
                        if let Ok(_) = addr.to_socket_addrs() {
                            settings.lobby_url = addr.clone();
                            pending_msgs.0.push(format!("[INFO] lobby_url has been set to {}", settings.lobby_url));
                        } else {
                            pending_msgs.0.push(format!("[FAIL] {addr} is not a valid URL (of type 127.0.0.1:9983)"));
                        }
                    }
                    SettingValue::LobbyMode(value) => {
                        settings.local_lobby = *value;
                        pending_msgs.0.push(format!("[INFO] local_lobby has been set to {}", settings.local_lobby));
                    }
                }
            }
        }
    }
}

impl TryFrom<String> for GameCommand {
    type Error = &'static str;
    fn try_from(mut value: String) -> Result<Self, Self::Error> {
        value.remove(0);
        let mut iter = value.split_whitespace();
        match iter.next() {
            Some(cmd) => {
                match cmd {
                    "toggle" => {
                        match iter.next() {
                            Some(setting) => {
                                match setting {
                                    "lobby_mode" => {
                                        return Ok(GameCommand::Toggle(SettingToggle::LobbyMode));
                                    }
                                    "music" | "music_enabled" => {
                                        return Ok(GameCommand::Toggle(SettingToggle::Music));
                                    }
                                    "sfx" | "sfx_enabled" => {
                                        return Ok(GameCommand::Toggle(SettingToggle::Sfx));
                                    }
                                    "hitboxes" | "hitboxes_enabled" => {
                                        return Ok(GameCommand::Toggle(SettingToggle::Hitboxes));
                                    }
                                    "debug" => {
                                        return Ok(GameCommand::Toggle(SettingToggle::Debug));
                                    }
                                    _ => {
                                        return Err("Invalid setting");
                                    }
                                }
                            }
                            None => {
                                return Err("Command 'toggle' requires a setting");
                            }
                        }
                    }
                    "set" => {
                        match iter.next() {
                            Some(setting) => {
                                match setting {
                                    "lobby_url" | "lobby_mode" => {
                                        match iter.next() {
                                            Some(value) => {
                                                if setting == "lobby_url" {
                                                    return Ok(GameCommand::Set(SettingValue::LobbyUrl(value.to_string())));
                                                } else {
                                                    match value {
                                                        "true" | "1" | "True" | "Local" | "local" => return Ok(GameCommand::Set(SettingValue::LobbyMode(true))),
                                                        "false" | "0" | "False" | "Normal" | "normal" => return Ok(GameCommand::Set(SettingValue::LobbyMode(false))),
                                                        _ => return Err("Value needs to be a boolean"),
                                                    }
                                                }
                                            }
                                            None => return Err("Command 'set' needs a value"),
                                        }
                                    }
                                    _ => {
                                        return Err("Invalid setting");
                                    }
                                }
                            }
                            None => {
                                return Err("Command 'set' requires a setting");
                            }
                        }
                    }
                    _ => {
                        return Err("Invalid command");
                    }
                }
            },
            None => {
                return Err("No command has been entered!");
            }
        }
    }
}

pub fn command_input(
    mut events: EventReader<KeyboardInput>,
    mut chat_input: Query<(&mut Text, &mut ChatInput)>,
    mut chat_type: ResMut<NextState<ChatType>>,
    mut pending_msgs: ResMut<PendingMessages>,
    mut commands: EventWriter<GameCommand>,
) {
    for event in events.read() {
        if event.state == ButtonState::Released {
            continue;
        }
        let (mut text, mut buffer) = chat_input.single_mut();

        match &event.logical_key {
            Key::Enter => {
                match GameCommand::try_from(buffer.0.to_string()) {
                    Ok(command) => {
                        commands.send(command);
                    }
                    Err(e) => pending_msgs.0.push(format!("[FAIL] {e}")),
                }
                buffer.0 = "".to_string();
                text.sections[0].value = "Start typing...".to_string();
                chat_type.set(ChatType::Normal);
                return;
            }
            Key::Space => {
                buffer.0.push(' ');
                text.sections[0].value = buffer.0.to_string();
            }
            Key::Backspace => {
                buffer.0.pop();
                text.sections[0].value = buffer.0.to_string();
                if buffer.0.is_empty() {
                    chat_type.set(ChatType::Normal);
                    return;
                }
            }
            Key::Character(character) => {
                buffer.0.push_str(character);
                text.sections[0].value = buffer.0.to_string();
            }
            _ => continue,
        }
    }
}
