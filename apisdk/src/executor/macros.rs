/// Internal macro
#[macro_export]
#[doc(hidden)]
macro_rules! _function_path {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        name.trim_end_matches("::f")
            .trim_end_matches("::{{closure}}")
    }};
}

/// Send request
///
/// # Examples
///
/// ### Option 1: Convert whole response to struct
///
/// ```
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct TypeOfResponse {}
///
/// let req = self.get("/path/api").await?;
/// let res: TypeOfResponse = send!(req).await?;
/// ```
///
/// ### Option 2: Use a `JsonExtractor` to get some fields from response
///
/// ```
/// use apisdk::{ApiError, ApiResult, JsonExtractor};
/// use serde::{Deserialize, DeserializeOwned};
/// use serde_json::Value;
///
/// struct MyExtractor {}
/// impl JsonExtractor for MyExtractor {
///     fn try_extract<T>(value: Value) -> ApiResult<T>
///     where
///         T: DeserializeOwned,
///     {
///         let mut value = value;
///         match value.get("data") {
///             Some(data) => serde_json::from_value(data.take()).map_err(|e| e.into()),
///             None => Err(ApiError::InvalidJson(value)),
///         }
///     }
/// }
///
/// #[derive(Deserialize)]
/// struct TypeOfData {}
///
/// let req = client.get("/path/api").await?;
/// let res: TypeOfData = send!(req, MyExtractor).await?;
/// ```
///
/// # JsonExtractor
///
/// `JsonExtractor` is used to build result from response.
///
/// We provides two built-in implementations:
/// - `WholePayload`
///     - return whole payload of response
/// - `CodeDataMessage`
///     - parse the payload of response as `{ code: i64, data: T, message: Option<String> }`
///     - ensure `code` is `0` and return `data` field
#[macro_export]
macro_rules! send {
    ($req:expr) => {
        $crate::internal::_send(
            $req,
            $crate::internal::RequestConfigurator::new($crate::_function_path!(), None, true),
        )
    };
    ($req:expr, $extractor:ty) => {
        async {
            use $crate::JsonExtractor;
            let result: serde_json::Value = $crate::internal::_send(
                $req,
                $crate::internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None,
                    <$extractor>::require_headers(),
                ),
            )
            .await?;
            <$extractor>::try_extract(result)
        }
    };
}

/// Internal macro
#[macro_export]
#[doc(hidden)]
macro_rules! _send_with {
    ($req:expr, $config:expr) => {
        $crate::internal::_send($req, $config.merge($crate::_function_path!(), true))
    };
    ($req:expr, $extractor:ty, $config:expr) => {
        async {
            use $crate::JsonExtractor;
            let result: serde_json::Value = $crate::internal::_send(
                $req,
                $config.merge($crate::_function_path!(), <$extractor>::require_headers()),
            )
            .await?;
            <$extractor>::try_extract(result)
        }
    };
}

/// Send the payload as JSON
///
/// # Examples
///
/// ```
/// let data = json!({
///     "key": "value"
/// });
/// let req = client.post("/path/api").await?;
/// let res: TypeOfResponse = send_json!(req, data).await?;
/// ```
///
/// Please reference `send` for more information
#[macro_export]
macro_rules! send_json {
    ($req:expr, $json:expr) => {
        $crate::internal::_send_json(
            $req,
            &($json),
            $crate::internal::RequestConfigurator::new($crate::_function_path!(), None, true),
        )
    };
    ($req:expr, $json:expr, $extractor:ty) => {
        async {
            use $crate::JsonExtractor;
            let result: serde_json::Value = $crate::internal::_send_json(
                $req,
                &($json),
                $crate::internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None,
                    <$extractor>::require_headers(),
                ),
            )
            .await?;
            <$extractor>::try_extract(result)
        }
    };
}

/// Internal macro
#[macro_export]
#[doc(hidden)]
macro_rules! _send_json_with {
    ($req:expr, $json:expr, $config:expr) => {
        $crate::internal::_send_json(
            $req,
            &($json),
            $config.merge($crate::_function_path!(), true),
        )
    };
    ($req:expr, $json:expr, $extractor:ty, $config:expr) => {
        async {
            use $crate::JsonExtractor;
            let result: serde_json::Value = $crate::internal::_send_json(
                $req,
                &($json),
                $config.merge($crate::_function_path!(), <$extractor>::require_headers()),
            )
            .await?;
            <$extractor>::try_extract(result)
        }
    };
}

/// Send the payload as form
///
/// # Examples
///
/// ### Use HashMap to build form
///
/// ```
/// let mut form = HashMap::new();
/// form.insert("key", "value");
/// let req = client.post("/path/api").await?;
/// let res: TypeOfResponse = send_form!(req, form).await?;
/// ```
///
/// ### Use DynamicForm to build form
///
/// ```
/// use apisdk::DynamicForm;
///
/// let mut form = DynamicForm::new();
/// form.text("key", "value");
/// form.pair("part", Part::text("part-value"));
/// let req = client.post("/path/api").await?;
/// let res: TypeOfResponse = send_form!(req, form).await?;
/// ```
///
/// Please reference `send` for more information
#[macro_export]
macro_rules! send_form {
    ($req:expr, $form:expr) => {
        $crate::internal::_send_form(
            $req,
            $form,
            $crate::internal::RequestConfigurator::new($crate::_function_path!(), None, true),
        )
    };
    ($req:expr, $form:expr, $extractor:ty) => {
        async {
            use $crate::JsonExtractor;
            let result: serde_json::Value = $crate::internal::_send_form(
                $req,
                $form,
                $crate::internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None,
                    <$extractor>::require_headers(),
                ),
            )
            .await?;
            <$extractor>::try_extract(result)
        }
    };
}

/// Internal macro
#[macro_export]
#[doc(hidden)]
macro_rules! _send_form_with {
    ($req:expr, $form:expr, $config:expr) => {
        $crate::internal::_send_form($req, $form, $config.merge($crate::_function_path!(), true))
    };
    ($req:expr, $form:expr, $extractor:ty, $config:expr) => {
        async {
            use $crate::JsonExtractor;
            let result: serde_json::Value = $crate::internal::_send_form(
                $req,
                $form,
                $config.merge($crate::_function_path!(), <$extractor>::require_headers()),
            )
            .await?;
            <$extractor>::try_extract(result)
        }
    };
}

/// Send the payload as multipart form
///
/// ### Use MultipartForm to build form
///
/// ```
/// use apisdk::MultipartForm;
///
/// let mut form = MultipartForm::new();
/// form.text("key", "value");
/// form.pair("part", Part::text("part-value"));
/// let req = client.post("/path/api").await?;
/// let res: TypeOfResponse = send_multipart!(req, form).await?;
/// ```
///
/// Please reference `send` for more information
#[macro_export]
macro_rules! send_multipart {
    ($req:expr, $form:expr) => {
        $crate::internal::_send_multipart(
            $req,
            $form,
            $crate::internal::RequestConfigurator::new($crate::_function_path!(), None, true),
        )
    };
    ($req:expr, $form:expr, $extractor:ty) => {
        async {
            use $crate::JsonExtractor;
            let result: serde_json::Value = $crate::internal::_send_multipart(
                $req,
                $form,
                $crate::internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None,
                    <$extractor>::require_headers(),
                ),
            )
            .await?;
            <$extractor>::try_extract(result)
        }
    };
}

/// Internal macro
#[macro_export]
#[doc(hidden)]
macro_rules! _send_multipart_with {
    ($req:expr, $form:expr, $config:expr) => {
        $crate::internal::_send_multipart(
            $req,
            $form,
            $config.merge($crate::_function_path!(), true),
        )
    };
    ($req:expr, $form:expr, $extractor:ty, $config:expr) => {
        async {
            use $crate::JsonExtractor;
            let result: serde_json::Value = $crate::internal::_send_multipart(
                $req,
                $form,
                $config.merge($crate::_function_path!(), <$extractor>::require_headers()),
            )
            .await?;
            <$extractor>::try_extract(result)
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_trim_tail() {
        let s = "module::file::{{closure}}::{{closure}}";
        let o = s.trim_end_matches("::{{closure}}");
        println!("{o}");
    }
}
