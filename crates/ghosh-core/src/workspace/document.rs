use super::model::{Manifest, invalid, validate_name};
use crate::contracts::{AppError, ErrorCode, Result};
use toml_edit::{DocumentMut, Item};

pub struct Document {
    pub ast: DocumentMut,
}
impl Document {
    pub fn parse(text: &str) -> Result<Self> {
        if text.len() > 1_048_576 {
            return Err(invalid());
        }
        let ast = text.parse::<DocumentMut>().map_err(|_| invalid())?;
        reject_inline_secrets(ast.as_item())?;
        Ok(Self { ast })
    }
    pub fn schema(&self) -> Result<u32> {
        self.ast
            .get("schema")
            .and_then(Item::as_integer)
            .and_then(|v| u32::try_from(v).ok())
            .filter(|v| *v > 0)
            .ok_or_else(invalid)
    }
    pub fn manifest(&self) -> Result<Manifest> {
        let model: Manifest =
            toml_edit::de::from_document(self.ast.clone()).map_err(|_| invalid())?;
        model.validate()?;
        validate_supported_options(&self.ast)?;
        Ok(model)
    }
    pub fn rename(&mut self, name: &str) -> Result<()> {
        validate_name(name)?;
        let current = self
            .ast
            .get_mut("workspace")
            .and_then(Item::as_table_mut)
            .and_then(|t| t.get_mut("name"))
            .and_then(Item::as_value_mut)
            .ok_or_else(invalid)?;
        let decor = current.decor().clone();
        *current = toml_edit::Value::from(name);
        *current.decor_mut() = decor;
        Ok(())
    }
    pub fn text(&self) -> String {
        self.ast.to_string()
    }
}
fn validate_supported_options(ast: &DocumentMut) -> Result<()> {
    for (section, field, allowed) in [
        ("secrets", "backend", &["keychain"][..]),
        ("ui", "theme", &["dark", "light", "system"][..]),
    ] {
        if let Some(v) = ast.get(section).and_then(|t| t.get(field))
            && !v.as_str().is_some_and(|s| allowed.contains(&s))
        {
            return Err(invalid());
        }
    }
    if let Some(ssh) = ast.get("defaults").and_then(|d| d.get("ssh")) {
        for (key, max) in [
            ("port", 65535),
            ("connect_timeout_s", 3600),
            ("concurrency", 128),
        ] {
            if let Some(v) = ssh.get(key)
                && !v.as_integer().is_some_and(|n| n > 0 && n <= max)
            {
                return Err(invalid());
            }
        }
        if let Some(v) = ssh.get("known_hosts")
            && !matches!(v.as_str(), Some("workspace" | "system"))
        {
            return Err(invalid());
        }
    }
    if let Some(script) = ast.get("defaults").and_then(|d| d.get("script")) {
        if let Some(v) = script.get("runtime")
            && !matches!(v.as_str(), Some("js" | "python" | "node"))
        {
            return Err(invalid());
        }
        if let Some(v) = script.get("permissions")
            && !matches!(v.as_str(), Some("ask" | "deny" | "allow-listed"))
        {
            return Err(invalid());
        }
        if let Some(v) = script.get("timeout_s")
            && !v.as_integer().is_some_and(|n| n > 0 && n <= 86400)
        {
            return Err(invalid());
        }
    }
    Ok(())
}
fn reject_inline_secrets(item: &Item) -> Result<()> {
    if let Some(table) = item.as_table_like() {
        for (key, value) in table.iter() {
            reject_secret_key(key)?;
            reject_inline_secrets(value)?;
        }
    }
    if let Some(tables) = item.as_array_of_tables() {
        for table in tables {
            for (key, value) in table {
                reject_secret_key(key)?;
                reject_inline_secrets(value)?;
            }
        }
    }
    if let Some(value) = item.as_value() {
        reject_secret_value(value)?;
    }
    Ok(())
}
fn reject_secret_key(key: &str) -> Result<()> {
    if matches!(
        key,
        "password" | "private_key" | "token" | "credentials" | "vless_uri"
    ) {
        return Err(AppError::new(ErrorCode::InvalidDocument));
    }
    Ok(())
}
fn reject_secret_value(value: &toml_edit::Value) -> Result<()> {
    if let Some(array) = value.as_array() {
        for value in array {
            reject_secret_value(value)?;
        }
    }
    if let Some(table) = value.as_inline_table() {
        for (key, value) in table {
            reject_secret_key(key)?;
            reject_secret_value(value)?;
        }
    }
    Ok(())
}
