use erreur::*;
use svarog_algo_flat::{
    frost::{KeystoreSchnorr, SignatureSchnorr},
    gg18::{KeystoreEcdsa, SignatureEcdsa},
};
use svarog_grpc::{Algorithm, CoefComs, Keystore, Signature};

pub(crate) trait KeystoreConversion {
    fn to_proto(&self) -> Resultat<Keystore>;
    fn from_proto(keystore_pb: &Keystore) -> Resultat<Self>
    where
        Self: Sized;
}

impl KeystoreConversion for KeystoreEcdsa {
    fn to_proto(&self) -> Resultat<Keystore> {
        let mut keystore_pb = Keystore::default();
        keystore_pb.i = self.i as u64;
        keystore_pb.ui = self.ui.to_bytes().to_vec();
        keystore_pb.xi = self.xi.to_bytes().to_vec();
        for (i, coef_com_vec) in self.vss_scheme.iter() {
            let mut coef_com_vec_pb = CoefComs::default();
            for coef_com in coef_com_vec.iter() {
                let coef_com_pub = coef_com.to33bytes().to_vec();
                coef_com_vec_pb.values.push(coef_com_pub);
            }
            keystore_pb.vss_scheme.insert(*i as u64, coef_com_vec_pb);
        }
        keystore_pb.xpub = self.xpub().catch_()?;
        let misc = (self.paillier_key.clone(), self.paillier_n_dict.clone());
        let misc_bytes = serde_pickle::to_vec(&misc, Default::default()).catch_()?;
        keystore_pb.algo = Algorithm::Gg18Secp256k1.into();
        keystore_pb.misc = misc_bytes;

        Ok(keystore_pb)
    }

    fn from_proto(keystore_pb: &Keystore) -> Resultat<Self>
    where
        Self: Sized,
    {
        use svarog_algo_flat::k256::{ProjectivePoint, Scalar};
        assert_throw!(keystore_pb.algo() == Algorithm::Gg18Secp256k1);

        let mut keystore = Self::default();
        keystore.i = keystore_pb.i as usize;
        keystore.ui = Scalar::from_bytes_mod_order(&keystore_pb.ui);
        keystore.xi = Scalar::from_bytes_mod_order(&keystore_pb.xi);
        for (i, coef_com_vec_pb) in keystore_pb.vss_scheme.iter() {
            let mut coef_com_vec = Vec::new();
            for coef_com in coef_com_vec_pb.values.iter() {
                let coef_com = ProjectivePoint::from33bytes(coef_com).catch_()?;
                coef_com_vec.push(coef_com);
            }
            keystore.vss_scheme.insert(*i as usize, coef_com_vec);
        }
        (keystore.paillier_key, keystore.paillier_n_dict) =
            serde_pickle::from_slice(&keystore_pb.misc, Default::default()).catch_()?;
        keystore.paillier_key.precompute_cache().catch_()?;
        Ok(keystore)
    }
}

impl KeystoreConversion for KeystoreSchnorr {
    fn to_proto(&self) -> Resultat<Keystore> {
        let mut keystore_pb = Keystore::default();
        keystore_pb.i = self.i as u64;
        keystore_pb.ui = self.ui.to_bytes().to_vec();
        keystore_pb.xi = self.xi.to_bytes().to_vec();
        for (i, coef_com_vec) in self.vss_scheme.iter() {
            let mut coef_com_vec_pb = CoefComs::default();
            for coef_com in coef_com_vec.iter() {
                let coef_com_pub = coef_com.compress().to_bytes().to_vec();
                coef_com_vec_pb.values.push(coef_com_pub);
            }
            keystore_pb.vss_scheme.insert(*i as u64, coef_com_vec_pb);
        }
        keystore_pb.xpub = self.xpub().catch_()?;
        keystore_pb.algo = Algorithm::FrostEd25519.into();

        Ok(keystore_pb)
    }

    fn from_proto(keystore_pb: &Keystore) -> Resultat<Self>
    where
        Self: Sized,
    {
        use svarog_algo_flat::curve25519_dalek::{ristretto::CompressedRistretto, Scalar};
        assert_throw!(keystore_pb.algo() == Algorithm::FrostEd25519);

        let mut keystore = Self::default();
        keystore.i = keystore_pb.i as usize;
        assert_throw!(keystore_pb.ui.len() == 32);
        keystore.ui = Scalar::from_bytes_mod_order(keystore_pb.ui.as_slice().try_into().unwrap());
        assert_throw!(keystore_pb.xi.len() == 32);
        keystore.xi = Scalar::from_bytes_mod_order(keystore_pb.xi.as_slice().try_into().unwrap());
        for (i, coef_com_vec_pb) in keystore_pb.vss_scheme.iter() {
            let mut coef_com_vec = Vec::new();
            for coef_com in coef_com_vec_pb.values.iter() {
                assert_throw!(coef_com.len() == 32);
                let coef_com = CompressedRistretto::from_slice(&coef_com).catch_()?;
                let coef_com = coef_com.decompress().ifnone_()?;
                coef_com_vec.push(coef_com);
            }
            keystore.vss_scheme.insert(*i as usize, coef_com_vec);
        }

        Ok(keystore)
    }
}

pub(crate) trait SignatureConversion {
    fn to_proto(&self) -> Resultat<Signature>;
}

impl SignatureConversion for SignatureEcdsa {
    fn to_proto(&self) -> Resultat<Signature> {
        use svarog_algo_flat::k256::elliptic_curve::point::AffineCoordinates;

        let mut ret = Signature::default();
        ret.r = self.R.to_affine().x().to_vec();
        ret.s = self.s.to_bytes().to_vec();
        ret.v = self.v as u32;
        Ok(ret)
    }
}

impl SignatureConversion for SignatureSchnorr {
    fn to_proto(&self) -> Resultat<Signature> {
        let mut ret = Signature::default();
        ret.r = self.R.compress().to_bytes().to_vec();
        ret.s = self.s.to_bytes().to_vec();
        ret.v = 0;
        Ok(ret)
    }
}
