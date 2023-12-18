use std::{borrow::Cow, collections::HashMap};

use reqwest::multipart::{Form, Part};
use serde_json::Value;

/// This trait provides form related functions
pub trait FormLike {
    /// Check whether the form is a multipart form
    fn is_multipart(&self) -> bool;
    /// Get the meta of the form
    fn get_meta(&self) -> HashMap<String, String>;
    /// Treat the form as an urlencoded form
    fn get_form(self) -> Option<HashMap<String, String>>;
    /// Treat the form as a multipart form
    fn get_multipart(self) -> Option<Form>;
}

impl<K, V> FormLike for &[(K, V)]
where
    K: ToString,
    V: ToString,
{
    fn is_multipart(&self) -> bool {
        false
    }

    fn get_meta(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        for (k, v) in self.iter() {
            meta.insert(k.to_string(), v.to_string());
        }
        meta
    }

    fn get_form(self) -> Option<HashMap<String, String>> {
        let mut form = HashMap::new();
        for (k, v) in self.iter() {
            form.insert(k.to_string(), v.to_string());
        }
        Some(form)
    }

    fn get_multipart(self) -> Option<Form> {
        None
    }
}

impl FormLike for Value {
    fn is_multipart(&self) -> bool {
        false
    }

    fn get_meta(&self) -> HashMap<String, String> {
        (&self).get_meta()
    }

    fn get_form(self) -> Option<HashMap<String, String>> {
        (&self).get_form()
    }

    fn get_multipart(self) -> Option<Form> {
        None
    }
}

impl FormLike for &Value {
    fn is_multipart(&self) -> bool {
        false
    }

    fn get_meta(&self) -> HashMap<String, String> {
        match self {
            Value::Object(map) => {
                let mut meta = HashMap::new();
                for (k, v) in map {
                    meta.insert(k.to_string(), v.to_string());
                }
                meta
            }
            _ => HashMap::new(),
        }
    }

    fn get_form(self) -> Option<HashMap<String, String>> {
        match self {
            Value::Object(map) => {
                let mut form = HashMap::new();
                for (k, v) in map {
                    form.insert(k.to_string(), v.to_string());
                }
                Some(form)
            }
            _ => Some(HashMap::new()),
        }
    }

    fn get_multipart(self) -> Option<Form> {
        None
    }
}

impl<K, V> FormLike for HashMap<K, V>
where
    K: ToString,
    V: ToString,
{
    fn is_multipart(&self) -> bool {
        false
    }

    fn get_meta(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        for (k, v) in self.iter() {
            meta.insert(k.to_string(), v.to_string());
        }
        meta
    }

    fn get_form(self) -> Option<HashMap<String, String>> {
        let mut form = HashMap::new();
        for (k, v) in self.into_iter() {
            form.insert(k.to_string(), v.to_string());
        }
        Some(form)
    }

    fn get_multipart(self) -> Option<Form> {
        None
    }
}

impl<K, V> FormLike for &HashMap<K, V>
where
    K: ToString,
    V: ToString,
{
    fn is_multipart(&self) -> bool {
        false
    }

    fn get_meta(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        for (k, v) in self.iter() {
            meta.insert(k.to_string(), v.to_string());
        }
        meta
    }

    fn get_form(self) -> Option<HashMap<String, String>> {
        let mut form = HashMap::new();
        for (k, v) in self.iter() {
            form.insert(k.to_string(), v.to_string());
        }
        Some(form)
    }

    fn get_multipart(self) -> Option<Form> {
        None
    }
}

impl FormLike for Form {
    fn is_multipart(&self) -> bool {
        true
    }

    fn get_meta(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    fn get_form(self) -> Option<HashMap<String, String>> {
        None
    }

    fn get_multipart(self) -> Option<Form> {
        Some(self)
    }
}

/// Provides functions to update multipart form
pub trait MultipartFormOps {
    /// Add a data field with supplied name and value.
    fn text<T, U>(self, name: T, value: U) -> Self
    where
        T: Into<Cow<'static, str>>,
        U: Into<Cow<'static, str>>;

    /// Adds a customized Part.
    fn part<T>(self, name: T, part: Part) -> Self
    where
        T: Into<Cow<'static, str>>;
}

impl MultipartFormOps for Form {
    fn text<T, U>(self, name: T, value: U) -> Self
    where
        T: Into<Cow<'static, str>>,
        U: Into<Cow<'static, str>>,
    {
        self.text(name, value)
    }

    fn part<T>(self, name: T, part: Part) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        self.part(name, part)
    }
}

/// This struct wraps `reqwest::multipart::Form`
#[derive(Debug, Default)]
pub struct MultipartForm {
    meta: HashMap<String, String>,
    form: Form,
}

impl MultipartForm {
    pub fn new() -> Self {
        Self::default()
    }
}

impl FormLike for MultipartForm {
    fn is_multipart(&self) -> bool {
        true
    }

    fn get_meta(&self) -> HashMap<String, String> {
        self.meta.clone()
    }

    fn get_form(self) -> Option<HashMap<String, String>> {
        None
    }

    fn get_multipart(self) -> Option<Form> {
        Some(self.form)
    }
}

impl MultipartFormOps for MultipartForm {
    fn text<T, U>(self, name: T, value: U) -> Self
    where
        T: Into<Cow<'static, str>>,
        U: Into<Cow<'static, str>>,
    {
        let Self { mut meta, mut form } = self;
        let name = name.into();
        let value = value.into();
        meta.insert(name.to_string(), value.to_string());
        form = form.text(name, value);
        Self { meta, form }
    }

    fn part<T>(self, name: T, part: Part) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        let Self { mut meta, mut form } = self;
        let name = name.into();
        meta.insert(name.to_string(), format!("{:?}", part));
        form = form.part(name, part);
        Self { meta, form }
    }
}

/// The DynamicForm is mixin of urlencoded form and multipart form
#[derive(Debug, Default)]
pub struct DynamicForm<T = MultipartForm>
where
    T: FormLike + MultipartFormOps,
{
    map: HashMap<Cow<'static, str>, Cow<'static, str>>,
    form: Option<T>,
}

impl DynamicForm {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MultipartFormOps for DynamicForm {
    fn text<T, U>(self, name: T, value: U) -> Self
    where
        T: Into<Cow<'static, str>>,
        U: Into<Cow<'static, str>>,
    {
        let Self { mut map, form } = self;
        map.insert(name.into(), value.into());
        Self { map, form }
    }

    fn part<T>(self, name: T, part: Part) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        let Self { map, form } = self;
        let form = form.unwrap_or_default().part(name, part);
        Self {
            map,
            form: Some(form),
        }
    }
}

impl FormLike for DynamicForm {
    fn is_multipart(&self) -> bool {
        self.form.is_some()
    }

    fn get_meta(&self) -> HashMap<String, String> {
        let mut meta = self.form.as_ref().map(|f| f.get_meta()).unwrap_or_default();
        self.map.iter().for_each(|(k, v)| {
            meta.insert(k.to_string(), v.to_string());
        });
        meta
    }

    fn get_form(self) -> Option<HashMap<String, String>> {
        match self.form {
            Some(_) => None,
            None => {
                let mut form = HashMap::new();
                for (k, v) in self.map {
                    form.insert(k.to_string(), v.to_string());
                }
                Some(form)
            }
        }
    }

    fn get_multipart(self) -> Option<Form> {
        let mut form = self
            .form
            .unwrap_or_default()
            .get_multipart()
            .unwrap_or_default();
        for (k, v) in self.map {
            form = form.text(k, v);
        }
        Some(form)
    }
}
