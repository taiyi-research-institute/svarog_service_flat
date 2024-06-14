use erreur::*;
use svarog_algo::{
    elgamal_secp256k1::SignatureElgamal, schnorr_ed25519::SignatureSchnorr,
    schnorr_secp256k1::SignatureSchnorr as SignatureTaproot,
};
use svarog_grpc::{Algorithm, Curve, Scheme, Signature};
pub(crate) trait SignatureConversion {
    fn to_proto(&self) -> Resultat<Signature>;
}

impl SignatureConversion for SignatureElgamal {
    fn to_proto(&self) -> Resultat<Signature> {
        let mut ret = Signature::default();
        ret.r = SignatureElgamal::eval_rx(&self.R).to_bytes().to_vec();
        ret.s = self.s.to_bytes().to_vec();
        ret.v = self.v as u32;
        ret.algo = Some(Algorithm {
            curve: Curve::Secp256k1.into(),
            scheme: Scheme::ElGamal.into(),
        });
        Ok(ret)
    }
}

impl SignatureConversion for SignatureSchnorr {
    fn to_proto(&self) -> Resultat<Signature> {
        let mut ret = Signature::default();
        ret.r = self.R.compress().to_bytes().to_vec();
        ret.s = self.s.to_bytes().to_vec();
        ret.v = 0;
        ret.algo = Some(Algorithm {
            curve: Curve::Ed25519.into(),
            scheme: Scheme::Schnorr.into(),
        });
        Ok(ret)
    }
}

impl SignatureConversion for SignatureTaproot {
    fn to_proto(&self) -> Resultat<Signature> {
        use svarog_algo::k256::elliptic_curve::point::AffineCoordinates;

        let mut ret = Signature::default();
        ret.r = self.R.to_affine().x().to_vec();
        ret.s = self.s.to_bytes().to_vec();
        ret.v = 0;
        ret.algo = Some(Algorithm {
            curve: Curve::Ed25519.into(),
            scheme: Scheme::Schnorr.into(),
        });
        Ok(ret)
    }
}
