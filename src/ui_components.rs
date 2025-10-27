use gpui::{App, ParentElement, Pixels, Styled, Window};
use gpui_component::{
    IndexPath, StyledExt,
    label::Label,
    list::{List, ListDelegate, ListItem},
    tag::Tag,
};

pub struct NumberedListDelegate {
    pub items: Vec<String>,
    pub is_loading: bool,
}

impl ListDelegate for NumberedListDelegate {
    type Item = ListItem;

    fn items_count(&self, _section: usize, _cx: &App) -> usize {
        self.items.len()
    }

    fn render_item(
        &self,
        ix: IndexPath,
        _window: &mut Window,
        _cx: &mut gpui::Context<List<Self>>,
    ) -> Option<Self::Item> {
        self.items.get(ix.row).map(|item| {
            ListItem::new(ix)
                .child(
                    gpui::div()
                        .h_flex()
                        .gap_3()
                        .child(Tag::primary().w(Pixels::from(30.0)).child(
                            gpui::div().w_full().h_flex().justify_center().child(
                                Label::new(format!("{}", (ix.row + 1))).text_color(gpui::black()),
                            ),
                        ))
                        .child(Label::new(item.clone()).text_color(gpui::white())),
                )
                .disabled(true)
        })
    }
    fn loading(&self, _cx: &App) -> bool {
        self.is_loading
    }
    fn set_selected_index(
        &mut self,
        _: Option<IndexPath>,
        _window: &mut Window,
        cx: &mut gpui::Context<List<Self>>,
    ) {
        cx.notify();
    }
}
