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

// /// Internal macro
// #[macro_export]
// #[doc(hidden)]
// macro_rules! _is_trait {
//     ($type:ty, $trait:path) => {{
//         trait A {
//             fn is(&self) -> bool;
//         }

//         struct B<T: ?Sized>(core::marker::PhantomData<T>);

//         impl<T: ?Sized> core::ops::Deref for B<T> {
//             type Target = ();
//             fn deref(&self) -> &Self::Target {
//                 &()
//             }
//         }

//         impl<T: ?Sized> A for B<T>
//         where
//             T: $trait,
//         {
//             fn is(&self) -> bool {
//                 true
//             }
//         }

//         impl A for () {
//             fn is(&self) -> bool {
//                 false
//             }
//         }

//         B::<$type>(core::marker::PhantomData).is()
//     }};
// }

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
/// ### Option 2: Use a `Extractor` to get some fields from response
///
/// ```
/// use apisdk::{ApiError, ApiResult, Content, Extractor};
/// use serde::{Deserialize, DeserializeOwned};
///
/// struct MyExtractor {}
/// impl Extractor for MyExtractor {
///     fn try_extract<T>(content: Content) -> ApiResult<T>
///     where
///         T: DeserializeOwned,
///     {
///         match content {
///             Content::Json(mut value) => {
///                 match value.get("data") {
///                     Some(data) => serde_json::from_value(data.take()).map_err(|e| e.into()),
///                     None => Err(ApiError::InvalidJson(value)),
///                 }
///             }
///             Content::Text(text) => Err(ApiError::DecodeResponse("text/plain".to_string(), text)),
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
/// # Extractor
///
/// `Extractor` is used to build result from response.
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
        async {
            $crate::internal::_send(
                $req,
                $crate::internal::RequestConfigurator::new($crate::_function_path!(), None, true),
            )
            .await
            .and_then(|c| c.try_into())
        }
    };
    ($req:expr, ()) => {
        async {
            let _ = $crate::internal::_send(
                $req,
                $crate::internal::RequestConfigurator::new($crate::_function_path!(), None, false),
            )
            .await?;
            Ok(())
        }
    };
    ($req:expr, $extractor:ty) => {
        async {
            use $crate::Extractor;
            let result = $crate::internal::_send(
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
    ($req:expr, (), $config:expr) => {
        async {
            let _ = $crate::internal::_send($req, $config.merge($crate::_function_path!(), false))
                .await?;
            Ok(())
        }
    };
    ($req:expr, $extractor:ty, $config:expr) => {
        async {
            use $crate::Extractor;
            let result = $crate::internal::_send(
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
        async {
            $crate::internal::_send_json(
                $req,
                &($json),
                $crate::internal::RequestConfigurator::new($crate::_function_path!(), None, true),
            )
            .await
            .and_then(|c| c.try_into())
        }
    };
    ($req:expr, $json:expr, ()) => {
        async {
            let _ = $crate::internal::_send_json(
                $req,
                &($json),
                $crate::internal::RequestConfigurator::new($crate::_function_path!(), None, false),
            )
            .await?;
            Ok(())
        }
    };
    ($req:expr, $json:expr, $extractor:ty) => {
        async {
            use $crate::Extractor;
            let result = $crate::internal::_send_json(
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
    ($req:expr, $json:expr, (), $config:expr) => {
        async {
            let _ = $crate::internal::_send_json(
                $req,
                &($json),
                $config.merge($crate::_function_path!(), false),
            )
            .await?;
            Ok(())
        }
    };
    ($req:expr, $json:expr, $extractor:ty, $config:expr) => {
        async {
            use $crate::Extractor;
            let result = $crate::internal::_send_json(
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
        async {
            $crate::internal::_send_form(
                $req,
                $form,
                $crate::internal::RequestConfigurator::new($crate::_function_path!(), None, true),
            )
            .await
            .and_then(|c| c.try_into())
        }
    };
    ($req:expr, $form:expr, ()) => {
        async {
            let _ = $crate::internal::_send_form(
                $req,
                $form,
                $crate::internal::RequestConfigurator::new($crate::_function_path!(), None, false),
            )
            .await?;
            Ok(())
        }
    };
    ($req:expr, $form:expr, $extractor:ty) => {
        async {
            use $crate::Extractor;
            let result = $crate::internal::_send_form(
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
        async {
            $crate::internal::_send_form(
                $req,
                $form,
                $config.merge($crate::_function_path!(), true),
            )
            .await
            .and_then(|c| c.try_into())
        }
    };
    ($req:expr, $form:expr, (), $config:expr) => {
        async {
            let _ = $crate::internal::_send_form(
                $req,
                $form,
                $config.merge($crate::_function_path!(), false),
            )
            .await?;
            Ok(())
        }
    };
    ($req:expr, $form:expr, $extractor:ty, $config:expr) => {
        async {
            use $crate::Extractor;
            let result = $crate::internal::_send_form(
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
        async {
            $crate::internal::_send_multipart(
                $req,
                $form,
                $crate::internal::RequestConfigurator::new($crate::_function_path!(), None, true),
            )
            .await
            .and_then(|c| c.try_into())
        }
    };
    ($req:expr, $form:expr, ()) => {
        async {
            let _ = $crate::internal::_send_multipart(
                $req,
                $form,
                $crate::internal::RequestConfigurator::new($crate::_function_path!(), None, false),
            )
            .await?;
            Ok(())
        }
    };
    ($req:expr, $form:expr, $extractor:ty) => {
        async {
            use $crate::Extractor;
            let result = $crate::internal::_send_multipart(
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
        async {
            $crate::internal::_send_multipart(
                $req,
                $form,
                $config.merge($crate::_function_path!(), true),
            )
            .await
            .and_then(|c| c.try_into())
        }
    };
    ($req:expr, $form:expr, (), $config:expr) => {
        async {
            let _ = $crate::internal::_send_multipart(
                $req,
                $form,
                $config.merge($crate::_function_path!(), false),
            )
            .await?;
            Ok(())
        }
    };
    ($req:expr, $form:expr, $extractor:ty, $config:expr) => {
        async {
            use $crate::Extractor;
            let result = $crate::internal::_send_multipart(
                $req,
                $form,
                $config.merge($crate::_function_path!(), <$extractor>::require_headers()),
            )
            .await?;
            <$extractor>::try_extract(result)
        }
    };
}

/// Send and get raw response
#[macro_export]
macro_rules! send_raw {
    ($req:expr) => {
        $crate::internal::_send_raw(
            $req,
            $crate::internal::RequestConfigurator::new($crate::_function_path!(), None, false),
        )
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
