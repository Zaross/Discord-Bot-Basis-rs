use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use tracing::{info, warn};

pub struct Translator {
    locales: HashMap<String, HashMap<String, String>>,
    default_locale: String,
}

/// Loads all locale JSON files from a directory and builds a [`Translator`].
///
/// # Behavior
/// - Reads `*.json` files in `locales_dir`.
/// - Uses each file stem (e.g. `en` from `en.json`) as the locale key.
/// - Flattens and stores key/value translations per locale.
/// - Logs successful loads and warns on per-file failures.
/// - Fails if no locale files could be loaded.
///
/// # Default locale
/// - Uses `"en"` if present.
/// - Otherwise falls back to the first loaded locale.
///
/// # Errors
/// Returns an error if the directory cannot be read or if no locale maps are loaded.

/// Returns the translated string for `key` in `locale` with no variable substitution.
///
/// This is a convenience wrapper around [`Self::get_with`] with an empty variable list.

/// Returns the translated string for `key` in `locale`, applying `{name}`-style substitutions
/// from `vars`.
///
/// # Lookup order
/// 1. Normalized requested locale.
/// 2. Configured default locale.
/// 3. Fallback to the key itself (and emits a warning).
///
/// # Parameters
/// - `locale`: Requested locale identifier (normalized before lookup).
/// - `key`: Translation key.
/// - `vars`: Replacement pairs used by `substitute`.

/// Returns all loaded locale identifiers.
///
/// Note: Ordering is not guaranteed.

/// Returns `true` if a locale exists after normalization, otherwise `false`.
impl Translator {
    pub fn load(locales_dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let dir = locales_dir.as_ref();
        let mut locales: HashMap<String, HashMap<String, String>> = HashMap::new();

        let entries = std::fs::read_dir(dir)
            .map_err(|e| anyhow::anyhow!("Cannot read locales directory {:?}: {}", dir, e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            let lang = match path.file_stem().and_then(|s| s.to_str()) {
                Some(l) => l.to_string(),
                None => continue,
            };

            match load_json_flat(&path) {
                Ok(map) => {
                    info!("Loaded locale '{}' ({} keys)", lang, map.len());
                    locales.insert(lang, map);
                }
                Err(e) => {
                    warn!("Failed to load locale file {:?}: {}", path, e);
                }
            }
        }

        if locales.is_empty() {
            anyhow::bail!("No locale files found in {:?}", dir);
        }

        let default_locale = if locales.contains_key("en") {
            "en".to_string()
        } else {
            locales.keys().next().unwrap().clone()
        };

        info!(
            "Translation system ready. Default locale: '{}'. Loaded: {:?}",
            default_locale,
            locales.keys().collect::<Vec<_>>()
        );

        Ok(Self {
            locales,
            default_locale,
        })
    }

    pub fn get(&self, locale: &str, key: &str) -> String {
        self.get_with(locale, key, &[])
    }

    pub fn get_with(&self, locale: &str, key: &str, vars: &[(&str, &str)]) -> String {
        let lang = normalize_locale(locale);

        let raw = self
            .locales
            .get(lang)
            .and_then(|m| m.get(key))
            .or_else(|| {
                self.locales
                    .get(&self.default_locale)
                    .and_then(|m| m.get(key))
            })
            .cloned()
            .unwrap_or_else(|| {
                warn!("Missing translation key '{}' for locale '{}'", key, locale);
                key.to_string()
            });

        substitute(raw, vars)
    }

    pub fn available_locales(&self) -> Vec<&str> {
        self.locales.keys().map(String::as_str).collect()
    }

    pub fn has_locale(&self, locale: &str) -> bool {
        self.locales.contains_key(normalize_locale(locale))
    }
}

fn load_json_flat(path: &Path) -> anyhow::Result<HashMap<String, String>> {
    let content = std::fs::read_to_string(path)?;
    let value: Value = serde_json::from_str(&content)?;
    let mut map = HashMap::new();
    flatten_value("", &value, &mut map);
    Ok(map)
}

fn flatten_value(prefix: &str, value: &Value, out: &mut HashMap<String, String>) {
    match value {
        Value::Object(obj) => {
            for (k, v) in obj {
                let key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                flatten_value(&key, v, out);
            }
        }
        Value::String(s) => {
            out.insert(prefix.to_string(), s.clone());
        }
        other => {
            out.insert(prefix.to_string(), other.to_string());
        }
    }
}

fn substitute(mut s: String, vars: &[(&str, &str)]) -> String {
    for (name, value) in vars {
        s = s.replace(&format!("{{{}}}", name), value);
    }
    s
}

fn normalize_locale(locale: &str) -> &str {
    if let Some(short) = locale.split('-').next() {
        short
    } else {
        locale
    }
}

