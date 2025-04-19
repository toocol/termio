use super::ColorScheme;
use ahash::AHashMap;
use lazy_static::lazy_static;
use log::error;
use parking_lot::Mutex;
use rust_embed::Embed;

lazy_static! {
    static ref THEME_MGR: Mutex<ColorSchemeMgr> = Mutex::new(ColorSchemeMgr::default());
}

#[derive(Default)]
pub struct ColorSchemeMgr {
    inner: AHashMap<String, ColorScheme>,
}

impl ColorSchemeMgr {
    pub fn loads<T: Embed>(path: &'static str) {
        if let Some(file) = T::get(path) {
            let content = std::str::from_utf8(&file.data)
                .unwrap_or_default()
                .to_string();

            if let Ok(themes) = serde_json::from_str::<Vec<ColorScheme>>(&content) {
                for scheme in themes.into_iter() {
                    THEME_MGR.lock().inner.insert(scheme.name.clone(), scheme);
                }
            } else {
                error!("[ThemeMgr::loads] Convert `builtin_themes.json` to `Vec<Theme>` failed, check the file.");
            }
        };
    }

    #[inline]
    pub fn get(name: &str) -> Option<ColorScheme> {
        THEME_MGR.lock().inner.get(name).cloned()
    }
}
