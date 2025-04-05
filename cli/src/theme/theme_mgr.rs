use super::Theme;
use ahash::AHashMap;
use lazy_static::lazy_static;
use log::error;
use parking_lot::Mutex;
use rust_embed::Embed;

lazy_static! {
    static ref THEME_MGR: Mutex<ThemeMgr> = Mutex::new(ThemeMgr::default());
}

#[derive(Default)]
pub struct ThemeMgr {
    inner: AHashMap<String, Theme>,
}

impl ThemeMgr {
    pub fn loads<T: Embed>(path: &'static str) {
        if let Some(file) = T::get(path) {
            let content = std::str::from_utf8(&file.data)
                .unwrap_or_default()
                .to_string();

            if let Ok(themes) = serde_json::from_str::<Vec<Theme>>(&content) {
                for scheme in themes.into_iter() {
                    THEME_MGR.lock().inner.insert(scheme.name.clone(), scheme);
                }
            } else {
                error!("[ThemeMgr::loads] Convert `builtin_themes.json` to `Vec<Theme>` failed, check the file.");
            }
        };
    }

    #[inline]
    pub fn get(name: &str) -> Option<Theme> {
        THEME_MGR.lock().inner.get(name).cloned()
    }
}
