use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use tracing::{info, warn};

pub struct Translator {
    locales: HashMap<String, HashMap<String, String>>,
    default_locale: String,
}

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
            .or_else(|| self.locales.get(&self.default_locale).and_then(|m| m.get(key)))
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_translator() -> Translator {
        let dir = tempdir().unwrap();
        let en = dir.path().join("en.json");
        let de = dir.path().join("de.json");

        std::fs::write(
            &en,
            r#"{"ping": {"pong": "Pong! {ms}ms"}, "errors": {"generic": "Error"}}"#,
        )
        .unwrap();
        std::fs::write(
            &de,
            r#"{"ping": {"pong": "Pong! {ms}ms (de)"}}"#,
        )
        .unwrap();

        let path = dir.keep();
        Translator::load(path).unwrap()
    }

    #[test]
    fn basic_lookup() {
        let t = make_translator();
        assert_eq!(t.get("en", "errors.generic"), "Error");
    }

    #[test]
    fn substitution() {
        let t = make_translator();
        assert_eq!(
            t.get_with("en", "ping.pong", &[("ms", "42")]),
            "Pong! 42ms"
        );
    }

    #[test]
    fn fallback_to_default() {
        let t = make_translator();
        assert_eq!(t.get("de", "errors.generic"), "Error");
    }

    #[test]
    fn discord_locale_normalization() {
        let t = make_translator();
        assert_eq!(
            t.get_with("de-DE", "ping.pong", &[("ms", "5")]),
            "Pong! 5ms (de)"
        );
        assert_eq!(
            t.get_with("en-US", "ping.pong", &[("ms", "5")]),
            "Pong! 5ms"
        );
    }

    #[test]
    fn missing_key_returns_key() {
        let t = make_translator();
        assert_eq!(t.get("en", "totally.missing.key"), "totally.missing.key");
    }
}
