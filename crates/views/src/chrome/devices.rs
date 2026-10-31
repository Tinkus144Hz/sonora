use gpui::prelude::*;
use gpui::{App, ClickEvent, Context, Entity, SharedString, Window, div};
use i18n::t;
use music::connect::{Command, DeviceKind, Elsewhere};
use state::{Devices, Listening, Sonora};
use ui::{ActiveTheme as _, MenuItem, Picker, Popovers};

use crate::chrome::{jam, session};

/// The player bar's button for listening elsewhere and together: the account's other devices and
/// the Jam when the provider has a device network the listener keeps on, and Sonora Session on
/// every streaming provider. It draws nothing while no streaming provider is signed in.
pub(crate) struct DevicePicker {
    devices: Entity<Devices>,
    listening: Entity<Listening>,
    popovers: Popovers,
}

impl DevicePicker {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let devices = Sonora::global(cx).devices.clone();
        let listening = Sonora::global(cx).listening.clone();
        cx.observe(&devices, |_, _, cx| cx.notify()).detach();
        cx.observe(&listening, |_, _, cx| cx.notify()).detach();

        Self {
            devices,
            listening,
            popovers: Popovers::default(),
        }
    }

    /// The account's other devices, the controls of the one that plays, and the Jam.
    fn connect_items(&self, elsewhere: Option<&Elsewhere>, cx: &App) -> Vec<MenuItem> {
        let state = self.devices.read(cx);
        let devices = self.devices.clone();

        let mut items = vec![
            MenuItem::new("device-here", t!("devices-this-computer"))
                .icon(icon(DeviceKind::Computer))
                .selected(elsewhere.is_none())
                .on_click({
                    let devices = devices.clone();
                    move |_, _, cx| devices.read(cx).transfer_here()
                }),
        ];
        for device in state.devices() {
            let id = device.id.clone();
            let playing = elsewhere.is_some_and(|elsewhere| elsewhere.device.id == device.id);
            items.push(
                MenuItem::new(
                    SharedString::from(format!("device-{}", device.id)),
                    device.name.clone(),
                )
                .icon(icon(device.kind))
                .selected(playing)
                .on_click({
                    let devices = devices.clone();
                    move |_, _, cx| devices.read(cx).transfer(&id)
                }),
            );
        }

        if let Some(elsewhere) = elsewhere {
            let id = elsewhere.device.id.clone();
            let control = |command: Command| {
                let devices = devices.clone();
                let id = id.clone();
                move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
                    devices.read(cx).control(&id, command.clone())
                }
            };
            let (toggle, label, glyph) = match elsewhere.state.playing {
                true => (Command::Pause, t!("devices-pause"), "icons/pause.svg"),
                false => (Command::Play, t!("devices-resume"), "icons/play.svg"),
            };

            items.push(MenuItem::separator("device-separator"));
            items.push(
                MenuItem::new("device-previous", t!("player-previous"))
                    .icon("icons/skip-back.svg")
                    .on_click(control(Command::Previous)),
            );
            items.push(
                MenuItem::new("device-toggle", label)
                    .icon(glyph)
                    .on_click(control(toggle)),
            );
            items.push(
                MenuItem::new("device-next", t!("player-next"))
                    .icon("icons/skip-forward.svg")
                    .on_click(control(Command::Next)),
            );
        }

        items.push(MenuItem::separator("jam-separator"));
        match state.jam() {
            Some(jam) => {
                if let Some(link) = jam.link.clone() {
                    items.push(
                        MenuItem::new("jam-code", t!("devices-jam-code-menu"))
                            .icon("icons/qr-code.svg")
                            .on_click({
                                let devices = devices.clone();
                                move |_, _, cx| {
                                    devices.update(cx, |devices, cx| devices.show_code(true, cx))
                                }
                            }),
                    );
                    items.push(
                        MenuItem::new("jam-copy", t!("devices-jam-copy"))
                            .icon("icons/copy.svg")
                            .on_click(move |_, _, cx| jam::copy_link(&link, cx)),
                    );
                }
                items.push(
                    MenuItem::new(
                        "jam-leave",
                        match jam.owner {
                            true => t!("devices-jam-end"),
                            false => t!("devices-jam-leave"),
                        },
                    )
                    .icon("icons/log-out.svg")
                    .on_click({
                        let devices = devices.clone();
                        move |_, _, cx| devices.update(cx, |devices, cx| devices.leave(cx))
                    }),
                );
            }
            None => {
                items.push(
                    MenuItem::new("jam-start", t!("devices-jam-start"))
                        .icon("icons/users-round-collaboration.svg")
                        .on_click({
                            let devices = devices.clone();
                            move |_, _, cx| devices.update(cx, |devices, _| devices.start_jam())
                        }),
                );
                items.push(
                    MenuItem::new("jam-join", t!("devices-jam-join"))
                        .icon("icons/link.svg")
                        .on_click({
                            let devices = devices.clone();
                            move |_, _, cx| {
                                devices.update(cx, |devices, cx| devices.show_join(true, cx))
                            }
                        }),
                );
            }
        }
        items
    }

    /// Starting or joining a Sonora Session, or what to do with the one the listener is in.
    fn session_items(&self, cx: &App) -> Vec<MenuItem> {
        let listening = self.listening.clone();
        let state = listening.read(cx);
        let Some(room) = state.room() else {
            return vec![
                MenuItem::new("session-start", t!("devices-session-start"))
                    .icon("icons/headphones.svg")
                    .when(state.asking(), MenuItem::disabled)
                    .on_click({
                        let listening = listening.clone();
                        move |_, _, cx| listening.update(cx, |listening, cx| listening.start(cx))
                    }),
                MenuItem::new("session-join", t!("devices-session-join"))
                    .icon("icons/rectangle-ellipsis.svg")
                    .when(state.asking(), MenuItem::disabled)
                    .on_click(move |_, _, cx| {
                        listening.update(cx, |listening, cx| listening.show_join(true, cx))
                    }),
            ];
        };
        let code = room.code.clone();
        vec![
            MenuItem::new("session-code", t!("devices-session-code-menu"))
                .icon("icons/rectangle-ellipsis.svg")
                .on_click({
                    let listening = listening.clone();
                    move |_, _, cx| {
                        listening.update(cx, |listening, cx| listening.show_code(true, cx))
                    }
                }),
            MenuItem::new("session-copy", t!("devices-session-copy"))
                .icon("icons/copy.svg")
                .on_click(move |_, _, cx| session::copy_code(&code, cx)),
            MenuItem::new(
                "session-leave",
                match room.hosting() {
                    true => t!("devices-session-end"),
                    false => t!("devices-session-leave"),
                },
            )
            .icon("icons/log-out.svg")
            .on_click(move |_, _, cx| listening.update(cx, |listening, cx| listening.leave(cx))),
        ]
    }
}

impl Render for DevicePicker {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.devices.read(cx);
        let connect = state.available();
        let listening = self.listening.read(cx);
        if !connect && !listening.available(cx) {
            return div().into_any_element();
        }

        let theme = *cx.theme();
        let elsewhere = state.elsewhere().cloned().filter(|_| connect);
        let together = listening.room().is_some();
        let mut items = match connect {
            true => self.connect_items(elsewhere.as_ref(), cx),
            false => Vec::new(),
        };
        if !items.is_empty() {
            items.push(MenuItem::separator("session-separator"));
        }
        items.extend(self.session_items(cx));

        Picker::icon("devices", &self.popovers, "icons/monitor-speaker.svg")
            .tooltip_above(match connect {
                true => "player-devices",
                false => "player-listen-together",
            })
            .selected(elsewhere.is_some() || together)
            .tint(match elsewhere.is_some() || together {
                true => theme.foreground,
                false => theme.muted_foreground,
            })
            .width(Picker::WIDE)
            .items(items)
            .into_any_element()
    }
}

fn icon(kind: DeviceKind) -> &'static str {
    match kind {
        DeviceKind::Computer => "icons/monitor.svg",
        DeviceKind::Phone => "icons/smartphone.svg",
        DeviceKind::Tablet => "icons/tablet.svg",
        DeviceKind::Tv => "icons/tv.svg",
        DeviceKind::Speaker | DeviceKind::Console | DeviceKind::Car | DeviceKind::Other => {
            "icons/speaker.svg"
        }
    }
}
