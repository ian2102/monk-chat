#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use crate::egui::Color32;
use crate::egui::RichText;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_matchbox::prelude::*;
use chrono::Utc;
use rand::seq::SliceRandom;
use urlencoding;

const HOST: &str = "ws://127.0.0.1:3536";

#[derive(Clone, PartialEq)]
struct Member {
    id: String,
    color: Color32,
}

impl Member {
    fn new(id: String) -> Member {
        let colors = [
            Color32::DARK_GREEN,
            Color32::DARK_RED,
            Color32::RED,
            Color32::YELLOW,
            Color32::GREEN,
            Color32::BLUE,
            Color32::GOLD,
            Color32::LIGHT_RED,
            Color32::LIGHT_GREEN,
            Color32::DARK_BLUE,
        ];

        let mut rng = rand::thread_rng();
        let color = *colors.choose(&mut rng).unwrap();

        Member { id, color }
    }
}

#[derive(Clone, PartialEq)]
struct Message {
    member_id: String,
    content: String,
    date: String,
}

#[derive(Resource)]
struct AppState {
    rooms: Vec<Room>,
    input_room_url: String,
}

#[derive(Component)]
struct Room {
    is_window_open: bool,
    room_url: String,
    messages: Vec<Message>,
    input_text: String,
    socket: MatchboxSocket<SingleChannel>,
    name: String,
    members: Vec<Member>,
}

impl Room {
    fn new(name: &String) -> Self {
        let encoded_room_name = urlencoding::encode(name);

        Room {
            is_window_open: true,
            room_url: format!("{HOST}/{}", encoded_room_name),
            messages: Vec::new(),
            input_text: String::new(),
            socket: MatchboxSocket::from(
                WebRtcSocketBuilder::new(format!("{HOST}/{}", encoded_room_name))
                    .add_channel(ChannelConfig::reliable()),
            ),
            name: encoded_room_name.to_string(),
            members: vec![Member::new("You".to_string())],
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // fill the entire browser window
                fit_canvas_to_parent: true,
                // don't hijack keyboard shortcuts like F5, F6, F12, Ctrl+R etc.
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugin(EguiPlugin)
        .add_startup_system(setup)
        .add_systems((
            bevy::window::close_on_esc,
            handle_peers,
            accept_messages,
            ui_system,
        ))
        .insert_resource(AppState {
            rooms: Vec::new(),
            input_room_url: String::from(""),
        })
        .run();
}

fn ui_system(
    mut contexts: EguiContexts,
    mut app_state: ResMut<AppState>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let ctx = contexts.ctx_mut();
    let button_size = 20.0;

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            egui::menu::menu_button(ui, RichText::new("File").size(button_size), |ui| {
                if ui
                    .button(RichText::new("Preferences").size(button_size))
                    .clicked()
                {
                    // Open preferences dialog or perform related actions
                }
                if ui
                    .button(RichText::new("Settings").size(button_size))
                    .clicked()
                {
                    // Open settings dialog or perform related actions
                }
                if ui
                    .button(RichText::new("Wipe Session").size(button_size))
                    .clicked()
                {
                    // Open settings dialog or perform related actions
                }
                if ui.button(RichText::new("Quit").size(button_size)).clicked() {
                    std::process::exit(0);
                }
            });

            egui::menu::menu_button(ui, RichText::new("Network").size(button_size), |ui| {
                if ui
                    .button(RichText::new("Status").size(button_size))
                    .clicked()
                {
                    // Perform connect action
                }
                if ui
                    .button(RichText::new("Disconnect").size(button_size))
                    .clicked()
                {
                    // Perform disconnect action
                }
            });

            egui::menu::menu_button(ui, RichText::new("Help").size(button_size), |ui| {
                if ui
                    .button(RichText::new("Documentation").size(button_size))
                    .clicked()
                {
                    // Open documentation or perform related actions
                }
                if ui
                    .button(RichText::new("About").size(button_size))
                    .clicked()
                {
                    // Open about dialog or perform related actions
                }
                if ui
                    .button(RichText::new("Support").size(button_size))
                    .clicked()
                {
                    // Open support page or perform related actions
                }
            });
        });
    });

    egui::SidePanel::right("right_panel")
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.allocate_space(egui::Vec2::new(1.0, 10.0));
            ui.heading(RichText::new("Members").size(button_size));
            ui.allocate_space(egui::Vec2::new(1.0, 10.0));

            ui.vertical(|ui| {
                ui.separator();
                for room in app_state.rooms.iter_mut() {
                    if room.is_window_open {
                        for member in room.members.iter_mut() {
                            ui.label(
                                RichText::new(format!("{}", member.id))
                                    .color(member.color)
                                    .size(button_size),
                            );
                            ui.separator();
                        }
                    }
                }
            });
        });

    egui::SidePanel::left("left_panel")
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.allocate_space(egui::Vec2::new(1.0, 10.0));
            ui.heading(RichText::new("Join").size(button_size));
            ui.separator();

            ui.vertical(|ui| {
                ui.vertical(|ui| {
                    ui.allocate_space(egui::Vec2::new(1.0, 5.0));
                    ui.label(RichText::new("Room URL:").size(button_size));
                    ui.allocate_space(egui::Vec2::new(1.0, 5.0));
                    ui.text_edit_singleline(&mut app_state.input_room_url);

                    ui.allocate_space(egui::Vec2::new(1.0, 5.0));

                    if ui
                        .button(RichText::new("Connect").size(button_size))
                        .clicked()
                    {
                        let input_room = Room::new(&app_state.input_room_url.clone());

                        if app_state.input_room_url.is_empty() {
                            return;
                        }

                        if let Some(index) = app_state
                            .rooms
                            .iter()
                            .position(|room| room.room_url == input_room.room_url)
                        {
                            for (i, room) in app_state.rooms.iter_mut().enumerate() {
                                room.is_window_open = i == index;
                            }
                        } else {
                            for room in &mut app_state.rooms {
                                room.is_window_open = false;
                            }
                            app_state.rooms.push(input_room);
                        }

                        app_state.input_room_url.clear();
                    }
                });
            });

            ui.allocate_space(egui::Vec2::new(1.0, 30.0));

            ui.label(RichText::new("Rooms").size(button_size));
            ui.separator();
            let mut clicked_room_index: Option<usize> = None;

            for (index, room) in app_state.rooms.iter_mut().enumerate() {
                ui.allocate_space(egui::Vec2::new(1.0, 10.0));

                let clicked = ui
                    .button(RichText::new(&room.name).size(button_size))
                    .clicked();

                if clicked {
                    clicked_room_index = Some(index);
                }
                ui.allocate_space(egui::Vec2::new(1.0, 10.0));
            }

            if let Some(index) = clicked_room_index {
                for (i, room) in app_state.rooms.iter_mut().enumerate() {
                    room.is_window_open = i == index;
                }
            }
        });
    for room in app_state.rooms.iter_mut() {
        if room.is_window_open {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.allocate_space(egui::Vec2::new(1.0, 5.0));

                ui.label(
                    RichText::new(format!("Room: {}", room.name))
                        .size(40.0)
                        .underline(),
                );

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let reversed_messages: Vec<&Message> = room.messages.iter().rev().collect();
                        ui.allocate_space(egui::Vec2::new(1.0, 15.0));
                        for message in reversed_messages {
                            ui.allocate_space(egui::Vec2::new(1.0, 5.0));
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Content: ").size(button_size));
                                ui.label(
                                    RichText::new(format!("{}", message.content))
                                        .color(Color32::WHITE)
                                        .size(button_size),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Member: ").size(button_size));
                                let mut member_found = false;
                                for member in room.members.iter_mut() {
                                    if member.id == message.member_id {
                                        ui.label(
                                            RichText::new(format!("{}", member.id))
                                                .color(member.color)
                                                .size(button_size),
                                        );
                                        member_found = true;
                                        break;
                                    }
                                }
                                if !member_found {
                                    ui.label(RichText::new("Unknown").size(button_size));
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Date: ").size(button_size));
                                ui.label(
                                    RichText::new(format!("{}", message.date))
                                        .color(Color32::LIGHT_BLUE)
                                        .size(button_size),
                                );
                            });
                        }
                    });
                });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.horizontal(|ui| {
                        let re = ui.add(
                            egui::TextEdit::singleline(&mut room.input_text)
                                .desired_width(ui.available_width() / 1.2),
                        );

                        if ui.button("Send").clicked()
                            || re.lost_focus() && keyboard_input.pressed(KeyCode::Return)
                        {
                            if re.lost_focus() && keyboard_input.pressed(KeyCode::Return) {
                                re.request_focus()
                            }
                            if room.input_text.is_empty() {
                                return;
                            }

                            let member_id: String = "You".to_string();

                            let content = room.input_text.clone();
                            let date = Utc::now();
                            let message = Message {
                                member_id,
                                content,
                                date: date.to_string(),
                            };

                            let content = message.content.clone();
                            let packet = content.into_bytes().to_vec().into_boxed_slice();

                            let connected_peers: Vec<PeerId> =
                                room.socket.connected_peers().collect();
                            for peer in &connected_peers {
                                let cloned_packet = packet.clone();
                                room.socket.send(cloned_packet, *peer);
                            }

                            room.messages.push(message);
                            room.input_text.clear();
                        }
                    });
                });
            });
        }
    }
}

fn setup(mut commands: Commands, mut app_state: ResMut<AppState>) {
    commands.spawn(Camera2dBundle::default());

    let room_name = "monk-chat".to_string();

    let room = Room::new(&room_name);

    info!("connecting to matchbox server: {:?}", room.room_url);
    app_state.rooms.push(room);
}

fn handle_peers(mut app_state: ResMut<AppState>) {
    for room in app_state.rooms.iter_mut() {
        for (peer, state) in room.socket.update_peers() {
            match state {
                PeerState::Connected => {
                    info!("Peer joined: {:?}", peer);
                    let packet = "hello friend!".as_bytes().to_vec().into_boxed_slice();
                    room.socket.send(packet, peer);
                }
                PeerState::Disconnected => {
                    info!("Peer left: {:?}", peer);
                }
            }
        }
    }
}

fn accept_messages(mut app_state: ResMut<AppState>) {
    let date = Utc::now().to_string();

    for room in app_state.rooms.iter_mut() {
        for (peer, packet) in room.socket.receive() {
            let message = String::from_utf8_lossy(&packet);

            let member_id: String = peer.0.to_string();

            info!("Message from {:?}: {:?}", &member_id, message);

            let members = room.members.clone();
            let mut existing_member = false;
            for member in members {
                if member.id == member_id {
                    existing_member = true;
                    break;
                }
            }

            if !existing_member {
                room.members.push(Member::new(member_id.clone()));
            }

            let content = message.into_owned();
            if content != "hello friend!" {
                let message = Message {
                    member_id,
                    content,
                    date: date.to_string(),
                };
                room.messages.push(message);
            }
        }
    }
}
