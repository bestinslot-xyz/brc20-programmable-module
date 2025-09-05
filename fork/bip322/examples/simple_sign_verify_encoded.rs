use bip322::{sign_simple_encoded, verify_simple_encoded};

fn main() {
  let address = "bc1ppv609nr0vr25u07u95waq5lucwfm6tde4nydujnu8npg4q75mr5sxq8lt3";
  let message = "Hello World";
  let wif_private_key = "L3VFeEujGtevx9w18HD1fhRbCH67Az2dpCymeRE1SoPK6XQtaN2k";

  let base64_signature = sign_simple_encoded(address, message, wif_private_key).unwrap();

  assert!(verify_simple_encoded(address, message, &base64_signature).is_ok());
}
