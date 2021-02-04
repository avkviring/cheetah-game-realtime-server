use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};


pub mod grpc;
pub mod internal;
pub mod token;
fn main() {
    // let claims = Claim {
    //     user_id: "fio".to_string(),
    //     exp: 100000000000,
    // };
    // let token = encode(
    //     &Header::new(Algorithm::ES256),
    //     &claims,
    //     &EncodingKey::from_secret("secret".as_ref()),
    // )
    // .unwrap();
    //
    // println!("token {}", token);
    // let validation = Validation::default();
    // let decoded = jsonwebtoken::decode::<Claim>(
    //     &token,
    //     &DecodingKey::from_secret("secret".as_ref()),
    //     &validation,
    // );
    //
    // println!("decoded {:?}", decoded);
}
