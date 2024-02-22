use plonky2::{
    field::goldilocks_field::GoldilocksField, hash::hash_types::HashOut,
    plonk::config::GenericHashOut,
};
use serde::{Deserialize, Serialize};

type F = GoldilocksField;

#[derive(Debug, Clone, PartialEq)]
pub struct SerializedHashOut(pub HashOut<F>);

impl Serialize for SerializedHashOut {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let h = hex::encode(&self.0.to_bytes());
        serializer.serialize_str(&h)
    }
}

impl<'de> Deserialize<'de> for SerializedHashOut {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let h = hex::decode(s).map_err(serde::de::Error::custom)?;
        Ok(Self(HashOut::from_bytes(&h)))
    }
}

#[cfg(test)]
mod tests {
    use plonky2::{
        field::{goldilocks_field::GoldilocksField, types::Sample},
        hash::hash_types::HashOut,
    };

    use super::SerializedHashOut;

    type F = GoldilocksField;

    #[test]
    fn test_serialize_hashout() {
        let x = SerializedHashOut(HashOut::<F>::rand());
        let s = serde_json::to_string(&x).unwrap();
        let x_recovered: SerializedHashOut = serde_json::from_str(&s).unwrap();
        assert_eq!(x, x_recovered);
        println!("{}", &s);
    }
}
