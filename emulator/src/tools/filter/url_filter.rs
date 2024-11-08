use regex::Regex;
use std::{cell::RefCell, rc::Rc};
use tmui::prelude::*;
use tmui::tlib::object::{ObjectImpl, ObjectSubclass};

use super::regex_filter::{RegexFilter, RegexFilterHotSpot, RegexFilterHotSpotImpl};
use super::{
    BaseFilterImpl, Filter, FilterObject, HotSpotConstructer, HotSpotImpl, HotSpotType,
    EMAIL_ADDRESS_REGEX, FULL_URL_REGEX,
};

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UrlType {
    StandardUrl,
    Email,
    Unknown,
}

#[extends(Object)]
pub struct UrlFilterHotSpot {
    hotspot: Box<RegexFilterHotSpot>,
    url_object: RefCell<FilterObject>,
}
impl ObjectSubclass for UrlFilterHotSpot {
    const NAME: &'static str = "UrlFilterHotSpot";
}
impl ObjectImpl for UrlFilterHotSpot {}

impl UrlFilterHotSpot {
    pub fn url_type(&self) -> UrlType {
        let url = self.captured_texts().first();

        if let Some(url) = url {
            if FULL_URL_REGEX.is_match(url) {
                UrlType::StandardUrl
            } else if EMAIL_ADDRESS_REGEX.is_match(url) {
                UrlType::Email
            } else {
                UrlType::Unknown
            }
        } else {
            UrlType::Unknown
        }
    }
}
impl RegexFilterHotSpotImpl for UrlFilterHotSpot {
    fn set_captured_texts(&mut self, texts: Vec<String>) {
        self.hotspot.set_captured_texts(texts);
    }

    fn captured_texts(&self) -> &Vec<String> {
        self.hotspot.captured_texts()
    }
}
impl HotSpotConstructer for UrlFilterHotSpot {
    fn new(start_line: i32, start_column: i32, end_line: i32, end_column: i32) -> Box<Self> {
        let mut hotspot: Box<UrlFilterHotSpot> = Object::new(&[]);

        hotspot.hotspot = RegexFilterHotSpot::new(start_line, start_column, end_line, end_column);
        hotspot.set_type(HotSpotType::Link);

        let ptr = hotspot.as_mut() as *mut UrlFilterHotSpot as *mut dyn HotSpotImpl;
        hotspot.url_object.borrow_mut().set_filter(ptr);

        hotspot
    }
}
impl HotSpotImpl for UrlFilterHotSpot {
    fn initialize(&self) {
        self.url_object.borrow().activate();
    }

    #[inline]
    fn start_line(&self) -> i32 {
        self.hotspot.start_line()
    }

    #[inline]
    fn end_line(&self) -> i32 {
        self.hotspot.end_line()
    }

    #[inline]
    fn start_column(&self) -> i32 {
        self.hotspot.start_column()
    }

    #[inline]
    fn end_column(&self) -> i32 {
        self.hotspot.end_column()
    }

    #[inline]
    fn type_(&self) -> HotSpotType {
        self.hotspot.type_()
    }

    fn activate(&self, action: &str) {
        let mut url = self.captured_texts().first().unwrap().clone();
        let kind = self.url_type();
        if action == FilterObject::ACTION_COPY {
            // TODO: Save `url` to the system clipboard;
            return;
        }

        if action.is_empty()
            || action == FilterObject::ACTION_OPEN
            || action == FilterObject::ACTION_CLICK
        {
            match kind {
                UrlType::StandardUrl => {
                    if !url.contains("://") {
                        let mut new_url = "http://".to_string();
                        new_url.push_str(&url);
                        url = new_url;
                    }
                }
                UrlType::Email => {
                    let mut new_url = "mailto:".to_string();
                    new_url.push_str(&url);
                    url = new_url;
                }
                _ => {}
            }

            self.url_object
                .borrow()
                .emit_activated(url, action != FilterObject::ACTION_CLICK);
        }
    }

    fn set_type(&mut self, type_: HotSpotType) {
        self.hotspot.set_type(type_)
    }

    fn actions(&self) -> Vec<Action> {
        let mut list = vec![];
        let kind = self.url_type();

        assert!(kind == UrlType::StandardUrl || kind == UrlType::Email);

        match kind {
            UrlType::StandardUrl => {
                let open_action = Action::with_param(
                    self.url_object.borrow().action_open(),
                    vec!["Open link".to_value()],
                );
                let copy_action = Action::with_param(
                    self.url_object.borrow().action_open(),
                    vec!["Copy link address".to_value()],
                );
                list.push(open_action);
                list.push(copy_action);
            }
            UrlType::Email => {
                let open_action = Action::with_param(
                    self.url_object.borrow().action_open(),
                    vec!["Send email to...".to_value()],
                );
                let copy_action = Action::with_param(
                    self.url_object.borrow().action_copy(),
                    vec!["Copy email address".to_value()],
                );
                list.push(open_action);
                list.push(copy_action);
            }
            _ => {}
        }

        list
    }
}

/// A filter which matches URLs in blocks of text
pub struct UrlFilter {
    filter: RegexFilter,
}
impl UrlFilter {
    pub fn new() -> Self {
        Self {
            filter: RegexFilter::new(),
        }
    }
}
impl BaseFilterImpl for UrlFilter {
    fn add_hotspot(&mut self, hotspot: Box<dyn HotSpotImpl>) -> &dyn HotSpotImpl {
        self.filter.add_hotspot(hotspot)
    }

    fn get_line_column(&self, position: i32) -> (i32, i32) {
        self.filter.get_line_column(position)
    }
}
impl Filter for UrlFilter {
    #[inline]
    fn process(&mut self, regex: &Regex) {
        let mut pos;
        let text = self.buffer().borrow().to_string();
        assert!(!text.is_empty());

        let iter = regex.captures_iter(&text);
        for cap in iter {
            for i in 0..cap.len() {
                let matched = cap.get(i).unwrap();
                pos = matched.start() as i32;

                let (start_line, start_column) = self.get_line_column(pos);
                let (end_line, end_column) =
                    self.get_line_column(pos + matched.range().len() as i32);

                let mut spot =
                    UrlFilterHotSpot::new(start_line, start_column, end_line, end_column);
                let mut captured_texts = vec![];
                for matched in cap.iter().flatten() {
                    captured_texts.push(matched.as_str().to_string());
                }
                spot.set_captured_texts(captured_texts);

                let spot_ref = self.add_hotspot(spot);
                spot_ref.initialize();
            }
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.filter.reset()
    }

    #[inline]
    fn hotspot_at(&self, line: i32, column: i32) -> Option<Rc<Box<dyn HotSpotImpl>>> {
        self.filter.hotspot_at(line, column)
    }

    #[inline]
    fn hotspots(&self) -> &Vec<Rc<Box<dyn HotSpotImpl>>> {
        self.filter.hotspots()
    }

    #[inline]
    fn hotspots_at_line(&self, line: i32) -> Option<&Vec<Rc<Box<dyn HotSpotImpl>>>> {
        self.filter.hotspots_at_line(line)
    }

    #[inline]
    fn set_buffer(&mut self, buffer: Rc<RefCell<String>>, line_positions: Rc<RefCell<Vec<i32>>>) {
        self.filter.set_buffer(buffer, line_positions)
    }

    #[inline]
    fn buffer(&mut self) -> Rc<RefCell<String>> {
        self.filter.buffer()
    }
}
