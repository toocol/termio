use tmui::{
    prelude::{FromBytes, StaticType, ToBytes, ToValue},
    tlib::{
        events::KeyEvent,
        namespace::{KeyCode, KeyboardModifier},
        values::FromValue,
        Type, Value,
    },
};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct KeyPressedEvent {
    key_code: KeyCode,
    text: String,
    modifier: KeyboardModifier,
}

impl KeyPressedEvent {
    #[inline]
    pub fn new(key_code: KeyCode, text: String, modifier: KeyboardModifier) -> Self {
        Self {
            key_code,
            text,
            modifier,
        }
    }

    #[inline]
    pub fn key_code(&self) -> KeyCode {
        self.key_code
    }

    #[inline]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[inline]
    pub fn modifier(&self) -> KeyboardModifier {
        self.modifier
    }
}

impl StaticType for KeyPressedEvent {
    fn static_type() -> Type {
        Type::from_name("KeyPressedEvent")
    }

    fn bytes_len() -> usize {
        0
    }

    fn dyn_bytes_len(&self) -> usize {
        self.text.dyn_bytes_len() + KeyboardModifier::bytes_len() + KeyCode::bytes_len()
    }
}

impl ToBytes for KeyPressedEvent {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        let mut key_code = self.key_code.to_bytes();
        bytes.append(&mut key_code);

        let mut modifier = self.modifier.to_bytes();
        bytes.append(&mut modifier);

        let mut text = self.text.to_bytes();
        bytes.append(&mut text);

        bytes
    }
}

impl FromBytes for KeyPressedEvent {
    fn from_bytes(data: &[u8], _: usize) -> Self {
        let mut idx = 0;

        let key_code_len = KeyCode::bytes_len();
        let key_code_bytes = &data[idx..idx + key_code_len];
        let key_code = KeyCode::from_bytes(key_code_bytes, key_code_len);
        idx += key_code_len;

        let modifier_len = KeyboardModifier::bytes_len();
        let modifier_bytes = &data[idx..idx + modifier_len];
        let modifier = KeyboardModifier::from_bytes(modifier_bytes, modifier_len);
        idx += modifier_len;

        let text_bytes = &data[idx..];
        let text = String::from_bytes(text_bytes, 0);

        Self {
            key_code,
            text,
            modifier,
        }
    }
}

impl ToValue for KeyPressedEvent {
    fn to_value(&self) -> Value {
        Value::new(self)
    }

    fn value_type(&self) -> Type {
        Self::static_type()
    }
}

impl FromValue for KeyPressedEvent {
    fn from_value(value: &Value) -> Self {
        Self::from_bytes(value.data(), 0)
    }
}

pub trait ToKeyPressedEvent {
    fn to_key_pressed_event(&self) -> KeyPressedEvent;
}

impl ToKeyPressedEvent for KeyEvent {
    fn to_key_pressed_event(&self) -> KeyPressedEvent {
        KeyPressedEvent::new(self.key_code(), self.text().to_string(), self.modifier())
    }
}

#[cfg(test)]
mod tests {
    use tmui::tlib::events::EventType;

    use super::*;

    #[test]
    fn test_key_pressed_event_value() {
        let evt = KeyPressedEvent::new(
            KeyCode::Key5,
            "Hello".to_string(),
            KeyboardModifier::AltModifier,
        );
        let val = evt.to_value();
        assert_eq!(evt, val.get::<KeyPressedEvent>())
    }

    #[test]
    fn test_event_convert() {
        let evt1 = KeyEvent::new(EventType::KeyPress, KeyCode::KeyA, KeyboardModifier::AltModifier, "a");
        let evt2 = evt1.to_key_pressed_event();

        assert_eq!(evt1.key_code(), evt2.key_code());
        assert_eq!(evt1.text(), evt2.text());
        assert_eq!(evt1.modifier(), evt2.modifier());
    }
}
