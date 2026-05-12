use clerk_rs::{
	clerk::Clerk,
	validators::{authorizer::validate_jwt, jwks::MemoryCacheJwksProvider},
	ClerkConfiguration,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	env_logger::init();

	let secret_key = std::env::var("CLERK_SECRET_KEY").expect("CLERK_SECRET_KEY must be set");
	let token = std::env::args().nth(1).expect("Usage: jwt_debug <jwt-token>");

	// Decode and print header for debugging
	let parts: Vec<&str> = token.split('.').collect();
	if parts.len() >= 2 {
		use base64::engine::{general_purpose::URL_SAFE_NO_PAD, Engine};
		if let Ok(header_bytes) = URL_SAFE_NO_PAD.decode(parts[0]) {
			println!("Header: {}", String::from_utf8_lossy(&header_bytes));
		}
		if let Ok(payload_bytes) = URL_SAFE_NO_PAD.decode(parts[1]) {
			println!("Payload: {}", String::from_utf8_lossy(&payload_bytes));
		}
	}

	let config = ClerkConfiguration::new(None, None, Some(secret_key), None);
	let client = Clerk::new(config);
	let jwks_provider = Arc::new(MemoryCacheJwksProvider::new(client));

	println!("\nValidating JWT...");
	match validate_jwt(&token, jwks_provider).await {
		Ok(jwt) => {
			println!("SUCCESS: JWT validated");
			println!("  sub: {}", jwt.sub);
			println!("  iss: {}", jwt.iss);
			println!("  exp: {}", jwt.exp);
			println!("  version: {:?}", jwt.version);
			if let Some(ref org) = jwt.org {
				println!("  org_id: {}", org.id);
			}
			if let Some(ref org_v2) = jwt.org_v2 {
				println!("  org_v2.id: {}", org_v2.id);
			}
		}
		Err(e) => {
			println!("FAILED: {e:?}");
		}
	}

	Ok(())
}
