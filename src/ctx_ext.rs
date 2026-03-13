
pub trait CtxTranslate {
    fn t(&self, key: &str) -> String;

    fn tv(&self, key: &str, vars: &[(&str, &str)]) -> String;
}

impl<'a> CtxTranslate for crate::Context<'a> {
    fn t(&self, key: &str) -> String {
        let locale = self.locale().unwrap_or("en");
        self.data().translator.get(locale, key)
    }

    fn tv(&self, key: &str, vars: &[(&str, &str)]) -> String {
        let locale = self.locale().unwrap_or("en");
        self.data().translator.get_with(locale, key, vars)
    }
}
