use tmui::{
    input::text::Text,
    input::Input,
    label::Label,
    prelude::*,
    tlib::object::{ObjectImpl, ObjectSubclass},
    widget::WidgetImpl,
};

#[extends(Widget, Layout(HBox))]
#[derive(Childrenable)]
pub struct TextBundle {
    #[children]
    label: Tr<Label>,

    #[children]
    text: Tr<Text>,
}

impl ObjectSubclass for TextBundle {
    const NAME: &'static str = "TextBundle";
}

impl ObjectImpl for TextBundle {}

impl WidgetImpl for TextBundle {}

impl TextBundle {
    #[inline]
    pub fn new(label: &str) -> Tr<Self> {
        let mut tb = Self::new_alloc();
        tb.label.set_margin_top(3);
        tb.label.set_text(label);
        tb
    }

    #[inline]
    pub fn set_spacing(&mut self, spacing: i32) {
        self.text.set_margin_left(spacing);
    }

    #[inline]
    pub fn set_required(&mut self, required: bool) {
        self.text.set_required(required)
    }

    #[inline]
    pub fn check_required(&mut self) -> bool {
        self.text.check_required()
    }

    #[inline]
    pub fn value(&self) -> String {
        self.text.value()
    }
}
