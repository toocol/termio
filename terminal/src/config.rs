use once_cell::sync::Lazy;

pub struct Config {

}

#[inline]
fn instance() -> &'static mut Config {
    static mut CONFIG: Lazy<Config> = Lazy::new(|| Config::new());
    unsafe { &mut CONFIG }
}

impl Config {
    #[inline]
    fn new() -> Self {
        Config {  }
    }
}
