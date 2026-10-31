use gpui::prelude::*;
use gpui::{AnyElement, App, Context, div};
use i18n::t;
use state::{Outcome, ServerCheck, Sonora, Toasts};
use ui::{ActiveTheme as _, Button, Picker, Switch};

use super::{Group, SettingsView, Slot};

/// How much wider than a wide picker the address field stands, so a whole address fits.
const FIELD: f32 = 1.25;

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
        self.row(
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
    }

    /// The address of the listener's own server and the button that saves it. Saving takes only
    /// an address whose server answered its health check, and the button's tooltip says why
    /// one does not.
    pub(super) fn sessions_server_row(&self, cx: &mut Context<Self>) -> AnyElement {
        let typed = self.sessions_server.read(cx).text().trim().to_owned();
        let saved = self.settings.read(cx).sessions_server().trim() == typed;
        let listening = Sonora::global(cx).listening.read(cx);
        let answers = listening.checked() == Some(typed.as_str())
            && *listening.server_check() == ServerCheck::Answers;
        let hint = match listening.server_check() {
            _ if typed.is_empty() => "settings-sessions-save-empty",
            ServerCheck::Answers if saved => "settings-sessions-save-saved",
            ServerCheck::Answers => "settings-sessions-server-answers",
            ServerCheck::Idle | ServerCheck::Checking => "settings-sessions-server-checking",
            ServerCheck::Malformed => "settings-sessions-server-malformed",
            ServerCheck::Silent => "settings-sessions-server-silent",
        };

        self.row(
            t!("settings-sessions-server"),
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .w(Picker::WIDE * FIELD)
                        .child(self.sessions_server.clone()),
                )
                .child(
                    Button::new("sessions-server-save")
                        .secondary()
                        .h(cx.theme().metrics.field)
                        .label(t!("settings-sessions-save"))
                        .tooltip(hint)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if !answers || saved {
                                return;
                            }
                            let typed = typed.clone();
                            this.settings
                                .update(cx, |settings, cx| settings.set_sessions_server(typed, cx));
                            Toasts::show(Outcome::Done, "toast-sessions-server-saved", cx);
                        })),
                )
                .into_any_element(),
        )
    }
}
