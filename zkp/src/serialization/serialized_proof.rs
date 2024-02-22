use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    plonk::{
        circuit_data::CircuitData,
        config::GenericConfig,
        proof::{CompressedProofWithPublicInputs, ProofWithPublicInputs},
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub struct SerializedProof(pub Vec<u8>);

impl Serialize for SerializedProof {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let h = hex::encode(&self.0);
        serializer.serialize_str(&h)
    }
}

impl<'de> Deserialize<'de> for SerializedProof {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let h = hex::decode(s).map_err(serde::de::Error::custom)?;
        Ok(Self(h))
    }
}

impl SerializedProof {
    pub fn from_proof<F, C, const D: usize>(
        data: &CircuitData<F, C, D>,
        proof: &ProofWithPublicInputs<F, C, D>,
    ) -> Self
    where
        F: RichField + Extendable<D>,
        C: GenericConfig<D, F = F>,
    {
        let compressed_proof = proof
            .clone()
            .compress(&data.verifier_only.circuit_digest, &data.common)
            .unwrap();
        Self(compressed_proof.to_bytes())
    }

    pub fn to_proof<F, C, const D: usize>(
        &self,
        data: &CircuitData<F, C, D>,
    ) -> anyhow::Result<ProofWithPublicInputs<F, C, D>>
    where
        F: RichField + Extendable<D>,
        C: GenericConfig<D, F = F>,
    {
        let compressed_proof =
            CompressedProofWithPublicInputs::from_bytes(self.0.clone(), &data.common)?;
        let proof =
            compressed_proof.decompress(&data.verifier_only.circuit_digest, &data.common)?;
        Ok(proof)
    }
}

#[cfg(test)]
mod tests {
    use plonky2::{
        iop::witness::PartialWitness,
        plonk::{
            circuit_builder::CircuitBuilder, circuit_data::CircuitConfig,
            config::PoseidonGoldilocksConfig,
        },
    };

    use super::*;

    type C = PoseidonGoldilocksConfig;
    const D: usize = 2;
    type F = <C as GenericConfig<D>>::F;

    #[test]
    fn test_serialize_proof() {
        let builder = CircuitBuilder::<F, D>::new(CircuitConfig::default());
        let data = builder.build::<C>();
        let proof = data.prove(PartialWitness::new()).unwrap();
        let serialized_proof = SerializedProof::from_proof(&data, &proof);
        let recovered = serialized_proof.to_proof(&data).unwrap();
        assert_eq!(recovered, proof);

        let serialized_proof_str = serde_json::to_string(&serialized_proof).unwrap();
        let serialize_proof_recovered: SerializedProof =
            serde_json::from_str(&serialized_proof_str).unwrap();
        assert_eq!(serialize_proof_recovered, serialized_proof);
    }
}
