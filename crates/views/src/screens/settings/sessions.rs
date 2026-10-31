use gpui::prelude::*;
use gpui::{AnyElement, App, Context, div};
use i18n::t;
use state::{ServerCheck, Sonora};
use ui::{Picker, Switch};

use super::{Group, Row, SettingsView, Slot};

impl SettingsView {
    /// The Sessions group: whether the player bar shows the Sessions button, and which server
    /// Sonora Sessions go through. The address row stands only while the official server is off.
    pub(super) fn sessions_groups(&self, cx: &App) -> Vec<Group> {
        let mut slots = vec![Slot::SessionsButton, Slot::SessionsOfficial];
        if !self.settings.read(cx).sessions_official() {
            slots.push(Slot::SessionsServer);
        }
        vec![
            Group::new(
                "settings-group-sessions",
                "icons/users-round-collaboration.svg",
            )
            .rows(slots),
        ]
    }

    pub(super) fn sessions_button_row(&self, cx: &mut Context<Self>) -> AnyElement {
        let on = self.settings.read(cx).sessions_button();
        self.row(
            t!("settings-sessions-button"),
            Switch::new("sessions-button", on)
                .name(t!("settings-sessions-button"))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.settings
                        .update(cx, |settings, cx| settings.set_sessions_button(!on, cx));
                }))
                .into_any_element(),
        )
    }

    pub(super) fn sessions_official_row(&self, cx: &mut Context<Self>) -> AnyElement {
        let on = self.settings.read(cx).sessions_official();
        Row::new(
            t!("settings-sessions-official"),
            Switch::new("sessions-official", on)
                .name(t!("settings-sessions-official"))
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.settings
                        .update(cx, |settings, cx| settings.set_sessions_official(!on, cx));
                    // turning the official server off is asking for an address, so the field
                    // that takes it gets the keys
                    if on {
                        this.sessions_server
                            .update(cx, |input, cx| input.focus(window, cx));
                    }
                }))
                .into_any_element(),
        )
        .detail(t!("settings-sessions-official-detail"))
        .into_any_element()
    }

    /// The address of the listener's own server, with what its health check found under it.
    pub(super) fn sessions_server_row(&self, cx: &mut Context<Self>) -> AnyElement {
        let detail = match Sonora::global(cx).listening.read(cx).server_check() {
            ServerCheck::Idle => t!("settings-sessions-server-detail"),
            ServerCheck::Checking => t!("settings-sessions-server-checking"),
            ServerCheck::Answers => t!("settings-sessions-server-answers"),
            ServerCheck::Malformed => t!("settings-sessions-server-malformed"),
            ServerCheck::Silent => t!("settings-sessions-server-silent"),
        };
        Row::new(
            t!("settings-sessions-server"),
            div()
                .w(Picker::WIDE)
                .child(self.sessions_server.clone())
                .into_any_element(),
        )
        .detail(detail)
        .into_any_element()
    }
}
