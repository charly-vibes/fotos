/// OS keychain abstraction for API key storage.
///
/// Uses the keyring crate to store/retrieve API keys from:
/// - GNOME Keyring (Linux GNOME)
/// - KWallet (Linux KDE)
/// - Windows Credential Manager
use anyhow::Result;

const SERVICE_NAME: &str = "fotos";

/// Map a provider string to a keychain account name.
///
/// Named providers (e.g. `"anthropic"`) use `{provider}-api-key`.
/// Custom endpoints (`"endpoint:{id}"`) use `"endpoint-{id}"` (hyphens only,
/// since colons are unsafe in some keychain backends).
fn make_account(provider: &str) -> String {
    if let Some(id) = provider.strip_prefix("endpoint:") {
        format!("endpoint-{id}")
    } else {
        format!("{provider}-api-key")
    }
}

pub fn store_api_key(provider: &str, key: &str) -> Result<()> {
    let account = make_account(provider);
    let entry = keyring::Entry::new(SERVICE_NAME, &account)?;
    entry.set_password(key)?;
    Ok(())
}

pub fn get_api_key(provider: &str) -> Result<String> {
    let account = make_account(provider);
    let entry = keyring::Entry::new(SERVICE_NAME, &account)?;
    Ok(entry.get_password()?)
}

pub fn delete_api_key(provider: &str) -> Result<()> {
    let account = make_account(provider);
    let entry = keyring::Entry::new(SERVICE_NAME, &account)?;
    entry.delete_credential()?;
    Ok(())
}
