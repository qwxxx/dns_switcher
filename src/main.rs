mod dns;
mod types;
mod ui_components;
mod utils;

use std::{borrow::Cow, cell::RefCell, rc::Rc, time::Duration, vec};

use anyhow::{Result, anyhow};
use gpui::{
    AppContext, Application, AssetSource, Bounds, Edges, IntoElement, ParentElement, Pixels,
    Render, SharedString, Styled, Window, WindowBounds, WindowOptions,
};
use gpui_component::{
    ActiveTheme, Disableable, Icon, Root, Sizable, StyledExt, ThemeRegistry, button::Button,
    label::Label, list::List, switch::Switch,
};
use rust_embed::RustEmbed;
use tray_icon::{
    TrayIconBuilder,
    menu::{MenuEvent, MenuItem},
};
use types::{AppData, AppState, AppView};

use crate::types::{DNS_MAP, DnsType};
#[derive(RustEmbed)]
#[folder = "./assets"]
#[include = "icons/**"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Self::get(path)
            .map(|f| Some(f.data))
            .ok_or_else(|| anyhow!("could not find asset at path \"{path}\""))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| p.starts_with(path).then(|| p.into()))
            .collect())
    }
}

impl AppData {
    pub fn clear_dns(&mut self, cx: &gpui::Context<'_, Self>) {
        self.dns_list.clear();
        self.app_state = AppState::Success;
        dns::set_dns_to_system(&vec![]).unwrap();
        self.start_loading_dns(cx);
    }
    pub fn start_loading_dns(&mut self, cx: &gpui::Context<'_, Self>) {
        self.dns_list.clear();
        self.app_state = AppState::Loading;
        cx.spawn(async |weakself, app| {
            let result = app
                .background_spawn(async move { dns::get_dns_from_system() })
                .await;
            weakself
                .update(app, move |this, cx| {
                    match result {
                        Ok(dns_list) => {
                            this.dns_type = DnsType::Unknown;
                            this.dns_list = dns_list;
                            if this.dns_list == this.config.custom_dns.clone() {
                                this.dns_type = DnsType::Custom;
                            }
                            for (key, value) in DNS_MAP.iter() {
                                if *value == this.dns_list {
                                    this.dns_type = key.clone();
                                }
                            }
                            this.app_state = AppState::Success;
                        }
                        Err(_) => {
                            this.app_state = AppState::Error;
                        }
                    }
                    cx.activate(true);
                    cx.notify();
                })
                .unwrap();
        })
        .detach();
    }
    pub fn set_dns_type(&mut self, t: &DnsType, cx: &mut gpui::Context<'_, Self>) {
        let dns_list = match *t {
            DnsType::Custom => self.config.custom_dns.clone(),
            _ => DNS_MAP.get(&t).unwrap_or(&vec![]).clone(),
        };
        match dns::set_dns_to_system(&dns_list) {
            Ok(_) => {
                self.start_loading_dns(cx);
            }
            Err(_) => {
                self.app_state = AppState::Error;
                cx.notify();
            }
        }
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let model = cx.read_entity(&self.app_model, |e, _| -> AppData { e.clone() });
        let list_delegate = ui_components::NumberedListDelegate {
            items: model.dns_list.clone(),
            is_loading: false,
        };
        gpui::div()
            .v_flex()
            .paddings(Edges::all(Pixels::from(10.0)))
            .items_center()
            .bg(gpui::opaque_grey(0.1, 1.0))
            .text_color(gpui::white())
            .size_full()
            .justify_center()
            .child(
                gpui::div()
                    .flex_grow()
                    .w_full()
                    .h_flex()
                    .items_start()
                    .child(cx.new(|cx| List::new(list_delegate, window, cx).no_query()))
                    .child(
                        Button::new("clear-btn")
                            .large()
                            .icon(
                                Icon::empty()
                                    .path("icons/circle-x.svg")
                                    .text_color(cx.theme().foreground),
                            )
                            .disabled(model.app_state == AppState::Loading)
                            .on_click(cx.listener(|view, _, _, cx| {
                                cx.update_entity(&view.app_model, |model, cx| {
                                    model.clear_dns(cx);
                                    cx.notify();
                                });
                            })),
                    )
                    .child(
                        Button::new("refresh-btn")
                            .large()
                            .icon(
                                Icon::empty()
                                    .path("icons/refresh.svg")
                                    .text_color(cx.theme().foreground),
                            )
                            .disabled(model.app_state == AppState::Loading)
                            .on_click(cx.listener(|view, _, _, cx| {
                                cx.update_entity(&view.app_model, |model, cx| {
                                    model.start_loading_dns(cx);
                                    cx.notify();
                                });
                            })),
                    ),
            )
            .child(
                gpui::div()
                    .h_flex()
                    .gap_2()
                    .justify_center()
                    .child(if model.dns_type == DnsType::Unknown {
                        Label::new("Unknown")
                    } else {
                        Label::new("Google")
                    })
                    .child(
                        Switch::new("switch")
                            .disabled(model.app_state != AppState::Success)
                            .checked(model.dns_type == DnsType::Custom)
                            .on_click(cx.listener(|view, checked, _, cx| {
                                cx.update_entity(&view.app_model, move |model, cx| {
                                    model.set_dns_type(
                                        if *checked {
                                            &DnsType::Custom
                                        } else {
                                            &DnsType::Google
                                        },
                                        cx,
                                    );
                                });
                            })),
                    )
                    .child(Label::new(model.config.custom_dns_name)),
            )
    }
}

fn main() {
    let mut _tray_icon = Rc::new(RefCell::new(None));
    let tray_c = _tray_icon.clone();
    let app = Application::new().with_assets(Assets);
    let readed_config = match utils::load_custom_dns_from_config() {
        Some(config) => config,
        None => {
            std::process::exit(1);
        }
    };

    app.run(move |cx| {
        gpui_component::init(cx);
        let theme = ThemeRegistry::global(cx).default_dark_theme().clone();

        gpui_component::Theme::global_mut(cx).apply_config(&theme);

        let icon = utils::load_icon_from_assets(cx.asset_source(), "icons/tray_icon.png").unwrap();
        let quit_i = MenuItem::new("Quit", true, None);
        let tray_menu = tray_icon::menu::Menu::new();
        tray_menu.append(&quit_i).unwrap();
        tray_c.borrow_mut().replace(
            TrayIconBuilder::new()
                .with_tooltip("DNS Switcher")
                .with_icon(icon)
                .with_menu_on_left_click(false)
                .with_menu(Box::new(tray_menu))
                .build()
                .unwrap(),
        );

        let size = gpui::size(gpui::px(300.), gpui::px(200.));
        let model = cx.new(|_| AppData {
            config: readed_config,
            ..Default::default()
        });
        let app_view = cx.new(|cx| AppView::new(cx, &model));

        utils::hide_app_from_dock();

        cx.spawn(async move |app| {
            loop {
                while let Ok(event) = MenuEvent::receiver().try_recv() {
                    if *quit_i.clone().id() == event.id {
                        app.update(|app| {
                            app.quit();
                        })
                        .unwrap()
                    }
                }
                while let Ok(event) = tray_icon::TrayIconEvent::receiver().try_recv() {
                    match event {
                        tray_icon::TrayIconEvent::Click {
                            id: _,
                            position: pos,
                            rect: _,
                            button: _,
                            button_state: state,
                        } => {
                            if state == tray_icon::MouseButtonState::Up {
                                app.update(|app| {
                                    if app.windows().len() == 0 {
                                        /*let model = app.new(|_| AppData {
                                            ..Default::default()
                                        });
                                        let app_view = app.new(|cx| AppView::new(cx, &model));*/
                                        let point = gpui::point(
                                            Pixels::from(pos.x / 2.0) - size.width / 2.0,
                                            Pixels::from(pos.y / 2.0),
                                        );
                                        let options = WindowOptions {
                                            show: true,
                                            is_resizable: false,
                                            is_movable: false,
                                            window_bounds: Some(WindowBounds::Windowed(
                                                Bounds::new(point, size),
                                            )),
                                            titlebar: None,
                                            ..Default::default()
                                        };
                                        app.open_window(options, |window, app| {
                                            app.new(|cx| {
                                                return Root::new(
                                                    app_view.clone().into(),
                                                    window,
                                                    cx,
                                                );
                                            })
                                        })
                                        .unwrap();
                                        app.activate(true);
                                    } else {
                                        let window = app.windows()[0];
                                        app.update_window(window, |_, window, _| {
                                            window.remove_window();
                                        })
                                        .unwrap();
                                    }
                                })
                                .unwrap();
                            }
                        }
                        _ => {}
                    }
                }
                gpui::Timer::after(std::time::Duration::from_millis(100)).await;
            }
        })
        .detach();
    });
}
