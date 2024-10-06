use bevy::{input::{keyboard::{Key, KeyboardInput}, ButtonState}, prelude::*};

use crate::{ui::chat::{ChatInput, ChatType, PendingMessages}, Settings};

enum SettingToggle {
    LobbyMode,
}

enum SettingValue {
    LobbyMode(bool),
    LobbyUrl(String),
}

#[derive(Event)]
pub enum Command {
    Toggle(SettingToggle),
    Set(SettingValue),
}

pub fn execute_cmds(
    mut commands: EventReader<Command>,
    mut settings: ResMut<Settings>,
    mut pending_msgs: ResMut<PendingMessages>,
) {
    for command in commands.read() {
        match command {
            Command::Toggle(setting) => {
                match setting {
                    SettingToggle::LobbyMode => {
                        settings.local_lobby = !settings.local_lobby;
                        pending_msgs.0.push(format!("[INFO] LobbyMode has been set to {}", settings.local_lobby));
                    }
                }
            }
            Command::Set(setting) => {
                match setting {
                    SettingValue::LobbyUrl(addr) => {
                        settings.lobby_url = addr.clone();
                        pending_msgs.0.push(format!("[INFO] LobbyUrl has been set to {}", settings.lobby_url));
                    }
                    SettingValue::LobbyMode(value) => {
                        settings.local_lobby = *value;
                        pending_msgs.0.push(format!("[INFO] LobbyMode has been set to {}", settings.local_lobby));
                    }
                }
            }
        }
    }
}

impl TryFrom<String> for Command {
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
                                        return Ok(Command::Toggle(SettingToggle::LobbyMode));
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
                                                    return Ok(Command::Set(SettingValue::LobbyUrl(value.to_string())));
                                                } else {
                                                    match value {
                                                        "true" | "1" | "True" | "Local" | "local" => return Ok(Command::Set(SettingValue::LobbyMode(true))),
                                                        "false" | "0" | "False" | "Normal" | "normal" => return Ok(Command::Set(SettingValue::LobbyMode(false))),
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
    mut commands: EventWriter<Command>,
) {
    for event in events.read() {
        if event.state == ButtonState::Released {
            continue;
        }
        let (mut text, mut buffer) = chat_input.single_mut();

        match &event.logical_key {
            Key::Enter => {
                match Command::try_from(buffer.0.to_string()) {
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
