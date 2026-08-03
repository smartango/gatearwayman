use rand::rngs::OsRng;
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use rsa::RsaPrivateKey;

pub fn generate_rsa_keypair_2048() -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048)?;
    let public_key = private_key.to_public_key();

    let private_pem = private_key.to_pkcs8_pem(LineEnding::LF)?;
    let public_pem = public_key.to_public_key_pem(LineEnding::LF)?;

    Ok((private_pem.to_string(), public_pem))
}