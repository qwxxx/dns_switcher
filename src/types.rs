use std::{collections::HashMap, sync::LazyLock};

use gpui::{AppContext, Entity, EventEmitter};
use serde::{Deserialize, Serialize};

pub static DNS_MAP: LazyLock<HashMap<DnsType, Vec<String>>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(
        DnsType::Google,
        vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
    );
    map
});

#[derive(Default, PartialEq, Clone)]
pub enum AppState {
    #[default]
    Loading,
    Success,
    Error,
}

#[derive(Default, Clone, Hash, PartialEq, Eq)]
pub enum DnsType {
    #[default]
    Unknown,
    Google,
    Custom,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct DnsConfig {
    pub custom_dns: Vec<String>,
    pub custom_dns_name: String,
}

#[derive(Default, Clone)]
pub struct AppData {
    pub dns_list: Vec<String>,
    pub app_state: AppState,
    pub dns_type: DnsType,
    pub config: DnsConfig,
}

pub struct ListChangedEvent {}
impl EventEmitter<ListChangedEvent> for AppData {}

pub struct AppView {
    pub app_model: Entity<AppData>,
}
impl AppView {
    pub fn new(app: &mut gpui::App, model: &Entity<AppData>) -> AppView {
        app.new(|cx| {
            let entity_clone = model.clone();
            cx.update_entity(&entity_clone, |data, cx| {
                data.start_loading_dns(cx);
            });
            cx.subscribe(model, |_, _, _, cx| {
                println!("EVENT!");
                cx.notify();
            })
            .detach();
        });

        Self {
            app_model: model.clone(),
        }
    }
}
