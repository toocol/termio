use libs::constant::IPC_NAME;
use platform::layout::termio_view::TermioView;
use tmui::{
   prelude::*,
   application::Application,
   application_window::ApplicationWindow,
};

fn main() {
   log4rs::init_file("src/log4rs.yaml", Default::default()).unwrap();

   let app = Application::<(), ()>::shared_builder(IPC_NAME)
       .width(1280)
       .height(800)
       .transparent(true)
       .title("Termio")
       .build();

   app.connect_activate(build_ui);

   app.run();
}

fn build_ui(window: &mut ApplicationWindow) {
    window.child(TermioView::new())
}