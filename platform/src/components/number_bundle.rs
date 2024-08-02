use tmui::{
    input::{number::Number, Input},
    label::Label,
    prelude::*,
    tlib::object::{ObjectImpl, ObjectSubclass},
    widget::WidgetImpl,
};

#[extends(Widget, Layout(HBox))]
#[derive(Childrenable)]
pub struct NumberBundle {
    #[children]
    label: Box<Label>,

    #[children]
    number: Box<Number>,
}

impl ObjectSubclass for NumberBundle {
    const NAME: &'static str = "NumberBundle";
}

impl ObjectImpl for NumberBundle {}

impl WidgetImpl for NumberBundle {}

impl NumberBundle {
    #[inline]
    pub fn new(label: &str) -> Box<Self> {
        let mut nb: Box<Self> = Object::new(&[]);
        nb.label.set_margin_top(3);
        nb.label.set_text(label);
        nb
    }

    #[inline]
    pub fn set_spacing(&mut self, spacing: i32) {
        self.number.set_margin_left(spacing);
    }

    #[inline]
    pub fn set_required(&mut self, required: bool) {
        self.number.set_required(required)
    }

    #[inline]
    pub fn check_required(&mut self) -> bool {
        self.number.check_required()
    }

    #[inline]
    pub fn value(&self) -> String {
        self.number.value()
    }

    #[inline]
    pub fn val(&self) -> Option<f32> {
        self.number.val()
    }
    #[inline]
    pub fn set_val(&mut self, val: f32) {
        self.number.set_val(val)
    }

    #[inline]
    pub fn min(&self) -> Option<f32> {
        self.number.min()
    }
    #[inline]
    pub fn set_min(&mut self, min: f32) {
        self.number.set_min(min)
    }

    #[inline]
    pub fn max(&self) -> Option<f32> {
        self.number.max()
    }
    #[inline]
    pub fn set_max(&mut self, max: f32) {
        self.number.set_max(max)
    }

    #[inline]
    pub fn step(&self) -> f32 {
        self.number.step()
    }
    #[inline]
    pub fn set_step(&mut self, step: f32) {
        self.number.set_step(step)
    }

    #[inline]
    pub fn is_enable_spinner(&self) -> bool {
        self.number.is_enable_spinner()
    }
    #[inline]
    pub fn set_enable_spinner(&mut self, enable_spinner: bool) {
        self.number.set_enable_spinner(enable_spinner)
    }
}
