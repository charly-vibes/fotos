/// OS keychain abstraction for API key storage.
///
/// Uses the keyring crate to store/retrieve API keys from:
/// - GNOME Keyring (Linux GNOME)
/// - KWallet (Linux KDE)
/// - Windows Credential Manager
use anyhow::Result;

const SERVICE_NAME: &str = "fotos";

pub fn store_api_key(provider: &str, key: &str) -> Result<()> {
    let account = format!("{}-api-key", provider);
    let entry = keyring::Entry::new(SERVICE_NAME, &account)?;
    entry.set_password(key)?;
    Ok(())
}

pub fn get_api_key(provider: &str) -> Result<String> {
    let account = format!("{}-api-key", provider);
    let entry = keyring::Entry::new(SERVICE_NAME, &account)?;
    Ok(entry.get_password()?)
}

pub fn delete_api_key(provider: &str) -> Result<()> {
    let account = format!("{}-api-key", provider);
    let entry = keyring::Entry::new(SERVICE_NAME, &account)?;
    entry.delete_credential()?;
    Ok(())
}
