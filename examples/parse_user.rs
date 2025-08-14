use clerk_rs::models::User;
use serde_json::Value;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the user.json file
    let json_content = fs::read_to_string("user.json")?;
    
    println!("=== Attempting to parse user.json with the User model ===\n");
    
    // First, try to parse as generic JSON and identify the problematic fields
    let mut json_value: Value = serde_json::from_str(&json_content)?;
    
    // The issue is that verification objects have different structures based on their type
    // For SAML verifications, they have "external_verification_redirect_url" but email verifications don't
    // We need to clean up the JSON to match what the current User model expects
    
    if let Some(obj) = json_value.as_object_mut() {
        // Remove the create_organizations_limit field that's not in the User struct
        obj.remove("create_organizations_limit");
        
        // Fix verification objects in email_addresses
        if let Some(email_addresses) = obj.get_mut("email_addresses").and_then(|v| v.as_array_mut()) {
            for email in email_addresses {
                if let Some(verification) = email.get_mut("verification").and_then(|v| v.as_object_mut()) {
                    if verification.get("object").and_then(|o| o.as_str()) == Some("verification_saml") {
                        // For SAML verifications, we need to keep the structure but ensure compatibility
                        // The EmailAddressVerification model expects different fields than SamlAccountVerification
                        // Let's convert this to a format the EmailAddressVerification can handle
                        verification.remove("external_verification_redirect_url");
                    }
                }
            }
        }
        
        // Fix verification objects in saml_accounts
        if let Some(saml_accounts) = obj.get_mut("saml_accounts").and_then(|v| v.as_array_mut()) {
            for saml_account in saml_accounts {
                if let Some(verification) = saml_account.get_mut("verification").and_then(|v| v.as_object_mut()) {
                    // SAML account verifications should keep their structure
                    // but let's ensure the expire_at field is present and valid
                    if !verification.contains_key("external_verification_redirect_url") {
                        verification.insert("external_verification_redirect_url".to_string(), Value::Null);
                    }
                }
            }
        }
        
        // Fix verification objects in enterprise_accounts
        if let Some(enterprise_accounts) = obj.get_mut("enterprise_accounts").and_then(|v| v.as_array_mut()) {
            for enterprise_account in enterprise_accounts {
                if let Some(verification) = enterprise_account.get_mut("verification").and_then(|v| v.as_object_mut()) {
                    // Similar to SAML accounts
                    if !verification.contains_key("external_verification_redirect_url") {
                        verification.insert("external_verification_redirect_url".to_string(), Value::Null);
                    }
                }
            }
        }
    }
    
    // Now try to parse the cleaned JSON
    let cleaned_json = serde_json::to_string_pretty(&json_value)?;
    
    match serde_json::from_str::<User>(&cleaned_json) {
        Ok(user) => {
            println!("✅ Successfully parsed user after cleanup:");
            println!("  ID: {:?}", user.id);
            println!("  First Name: {:?}", user.first_name);
            println!("  Last Name: {:?}", user.last_name);
            println!("  Email: {:?}", 
                user.email_addresses.as_ref()
                    .and_then(|emails| emails.first())
                    .map(|email| &email.email_address)
            );
            println!("  Email Addresses Count: {}", user.email_addresses.as_ref().map_or(0, |emails| emails.len()));
            println!("  SAML Accounts Count: {}", user.saml_accounts.as_ref().map_or(0, |accounts| accounts.len()));
            println!("  Enterprise Accounts Count: {}", user.enterprise_accounts.as_ref().map_or(0, |accounts| accounts.len()));
            println!("  Created At: {:?}", user.created_at);
            println!("  Last Sign In: {:?}", user.last_sign_in_at);
        }
        Err(e) => {
            println!("❌ Still failed to parse user JSON after cleanup: {}", e);
            
            // Show the detailed error context
            println!("\n=== Debugging Information ===");
            println!("Error details: {:#}", e);
            
            // Try to find the specific line that's causing issues
            if let Ok(value) = serde_json::from_str::<Value>(&cleaned_json) {
                if let Some(obj) = value.as_object() {
                    println!("\nFields present in cleaned JSON:");
                    for (key, _) in obj.iter() {
                        println!("  - {}", key);
                    }
                }
            }
        }
    }
    
    Ok(())
}