use asset::Asset;
use tmui::{
    icons::{svg_icon::SvgIcon, svg_toggle_icon::SvgToggleIcon},
    prelude::*,
    tlib::object::{ObjectImpl, ObjectSubclass},
    widget::{callbacks::CallbacksRegister, WidgetImpl},
};

#[extends(Widget, Layout(HBox))]
#[derive(Childrenable)]
pub struct WinControlButtons {
    #[derivative(Default(value = "{
        let file = Asset::get(\"icons/minimize.svg\").unwrap();
        SvgIcon::from_bytes(file.data.as_ref())
    }"))]
    #[children]
    minimize: Box<SvgIcon>,

    #[derivative(Default(value = "{
        let maximize = Asset::get(\"icons/large.svg\").unwrap();
        let restore = Asset::get(\"icons/restore.svg\").unwrap();
        SvgToggleIcon::from_bytes(&[maximize.data.as_ref(), restore.data.as_ref()])
    }"))]
    #[children]
    maximize_restore: Box<SvgToggleIcon>,

    #[derivative(Default(value = "{
        let file = Asset::get(\"icons/close.svg\").unwrap();
        SvgIcon::from_bytes(file.data.as_ref())
    }"))]
    #[children]
    close: Box<SvgIcon>,
}

impl ObjectSubclass for WinControlButtons {
    const NAME: &'static str = "WinControlButtons";
}

impl ObjectImpl for WinControlButtons {
    fn initialize(&mut self) {
        self.set_vexpand(true);
        self.width_request(135);
        self.set_halign(Align::End);

        let background = self.background();
        const CTRL_BTN_GREY: Color = Color::grey_with(225);
        const CTRL_BTN_RED: Color = Color::from_rgb(245, 40, 40);

        self.minimize.width_request(45);
        self.minimize.height_request(30);
        self.minimize.callback_hover_in(|w| w.set_background(CTRL_BTN_GREY));
        self.minimize.callback_hover_out(move |w| w.set_background(background));
        self.minimize.callback_mouse_released(|w, _| w.window().minimize());

        self.maximize_restore.width_request(45);
        self.maximize_restore.height_request(30);
        self.maximize_restore.callback_hover_in(|w| w.set_background(CTRL_BTN_GREY));
        self.maximize_restore.callback_hover_out(move |w| w.set_background(background));
        self.maximize_restore.callback_mouse_released(|w, _| {
            let icon = w.downcast_mut::<SvgToggleIcon>().unwrap();
            match icon.current_icon() {
                0 => icon.window().maximize(),
                1 => icon.window().restore(),
                _ => unreachable!()
            }
        });
        self.maximize_restore.callback_window_maximized(|w| w.downcast_mut::<SvgToggleIcon>().unwrap().set_current_icon(1));
        self.maximize_restore.callback_window_restored(|w| w.downcast_mut::<SvgToggleIcon>().unwrap().set_current_icon(0));

        self.close.width_request(45);
        self.close.height_request(30);
        self.close.callback_hover_in(|w| w.set_background(CTRL_BTN_RED));
        self.close.callback_hover_out(move |w| w.set_background(background));
        self.close.callback_mouse_released(|w, _| w.window().close());
    }
}

impl WidgetImpl for WinControlButtons {}

impl WinControlButtons {
    #[inline]
    pub fn new() -> Box<Self> {
        Object::new(&[])
    }
}
