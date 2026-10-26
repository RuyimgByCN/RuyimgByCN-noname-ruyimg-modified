//! Pins the flat Serde surface independently of the functional macro registry.

use rule_api::{EffectRequest, PlayerId};
use serde::ser::{Error, Impossible, Serialize, SerializeStructVariant, Serializer};
use serde_json::{json, Value};

type WireShape = (u32, Vec<&'static str>);
type TestError = serde::de::value::Error;

struct VariantShape(WireShape);

impl SerializeStructVariant for VariantShape {
    type Ok = WireShape;
    type Error = TestError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        _value: &T,
    ) -> Result<(), Self::Error> {
        self.0 .1.push(key);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.0)
    }
}

struct EffectShape;

macro_rules! reject_scalar {
    ($($method:ident($ty:ty)),* $(,)?) => {$(
        fn $method(self, _value: $ty) -> Result<Self::Ok, Self::Error> {
            Err(TestError::custom("expected an effect variant"))
        }
    )*};
}

impl Serializer for EffectShape {
    type Ok = WireShape;
    type Error = TestError;
    type SerializeSeq = Impossible<WireShape, TestError>;
    type SerializeTuple = Impossible<WireShape, TestError>;
    type SerializeTupleStruct = Impossible<WireShape, TestError>;
    type SerializeTupleVariant = Impossible<WireShape, TestError>;
    type SerializeMap = Impossible<WireShape, TestError>;
    type SerializeStruct = Impossible<WireShape, TestError>;
    type SerializeStructVariant = VariantShape;

    reject_scalar! {
        serialize_bool(bool), serialize_i8(i8), serialize_i16(i16), serialize_i32(i32),
        serialize_i64(i64), serialize_u8(u8), serialize_u16(u16), serialize_u32(u32),
        serialize_u64(u64), serialize_f32(f32), serialize_f64(f64), serialize_char(char),
        serialize_str(&str), serialize_bytes(&[u8]),
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(TestError::custom("expected an effect variant"))
    }
    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok((index, vec![]))
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(TestError::custom("expected an effect variant"))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(TestError::custom("expected an effect variant"))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(TestError::custom("expected an effect variant"))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(TestError::custom("expected an effect variant"))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(TestError::custom("expected an effect variant"))
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(TestError::custom("expected an effect variant"))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(VariantShape((index, vec![])))
    }
}

#[test]
fn all_v322_effect_ordinals_fields_and_json_are_compatible() {
    // Captured from the original flat enum, never regenerated from the macro registry.
    let fixtures: Vec<(String, Vec<(String, Value)>)> =
        serde_json::from_str(include_str!("effect_wire_v322.json")).unwrap();
    assert_eq!(fixtures.len(), 815);
    for (ordinal, (name, fields)) in fixtures.iter().enumerate() {
        let json = if fields.is_empty() {
            json!(name)
        } else {
            json!({ name: fields.iter().cloned().collect::<serde_json::Map<_, _>>() })
        };
        let effect: EffectRequest =
            serde_json::from_value(json.clone()).unwrap_or_else(|err| panic!("{name}: {err}"));
        let (index, keys) = effect.serialize(EffectShape).unwrap();
        assert_eq!(index as usize, ordinal, "{name}");
        assert_eq!(
            keys,
            fields
                .iter()
                .map(|(key, _)| key.as_str())
                .collect::<Vec<_>>(),
            "{name}"
        );
        assert_eq!(serde_json::to_value(&effect).unwrap(), json, "{name}");
    }
}

#[test]
fn legacy_optional_fields_keep_their_defaults() {
    let damage: EffectRequest = serde_json::from_value(json!({
        "DealDamage": {"source": 1, "target": 2, "amount": 1, "is_lose_hp": false}
    }))
    .unwrap();
    assert!(matches!(
        damage,
        EffectRequest::DealDamage { nature: 0, .. }
    ));

    let recovery: EffectRequest = serde_json::from_value(json!({
        "RecoverHp": {"player": 2, "amount": 1}
    }))
    .unwrap();
    assert!(matches!(
        recovery,
        EffectRequest::RecoverHp {
            source: PlayerId::NONE,
            ..
        }
    ));
}
