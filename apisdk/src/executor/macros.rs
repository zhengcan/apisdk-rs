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
/// # Forms
///
/// - `send!(req)` -> `impl Future<Output = ApiResult<T>>`
///     - send the request, and parse response as json or xml based on response
/// - `send!(req, ())` -> `impl Future<Output = ApiResult<()>>`
///     - send the request, verify response status, then discard response
/// - `send!(req, Body)` -> `impl Future<Output = ApiResult<apisdk::ResponseBody>>`
///     - send the request, verify response status, and decode response body
/// - `send!(req, Json)` -> `impl Future<Output = ApiResult<T>>`
///     - send the request, parse response as json, then use serde_json to deserialize it
/// - `send!(req, Xml)` -> `impl Future<Output = ApiResult<T>>`
///     - send the request, parse response as xml, then use quick_xml to deserialize it
/// - `send!(req, Text)` -> `impl Future<Output = ApiResult<T>>`
///     - send the request, parse response as text, then use FromStr to deserialize it
/// - `send!(req, OtherType)` -> `impl Future<Output = ApiResult<T>>`
///     - send the request, parse response as json, and use `OtherType` as JsonExtractor
/// - `send!(req, Json<OtherType>)` -> `impl Future<Output = ApiResult<T>>`
///     - send the request, parse response as json, and use `OtherType` as JsonExtractor
///
/// ### Built-in JsonExtractors
///
/// - std::string::String
///     - treat whole payload as text output
/// - serde_json::Value
///     - treat whole payload as json output
/// - apisdk::WholePayload
///     - an alias of serde_json::Value
/// - apisdk::CodeDataMessage
///     - parse `{code, data, message}` json payload, verify `code`, and return `data` field
///
/// # Examples
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
#[macro_export]
macro_rules! send {
    ($req:expr) => {
        $crate::send!($req, $crate::Auto, ())
    };
    ($req:expr, ()) => {
        async {
            let _ = $crate::__internal::send(
                $req,
                $crate::__internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None::<bool>,
                    false,
                ),
            )
            .await?;
            Ok(())
        }
    };
    ($req:expr, Body) => {
        async {
            $crate::__internal::send(
                $req,
                $crate::__internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None::<bool>,
                    true,
                ),
            )
            .await
            .and_then(|c| c.try_into())
        }
    };
    ($req:expr, Json) => {
        $crate::send!($req, $crate::Json, ())
    };
    ($req:expr, Xml) => {
        $crate::send!($req, $crate::Xml, ())
    };
    ($req:expr, Text) => {
        $crate::send!($req, $crate::Text, ())
    };
    ($req:expr, $parser:ty, ()) => {
        async {
            let result = $crate::__internal::send(
                $req,
                $crate::__internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None::<bool>,
                    false,
                ),
            )
            .await?;
            <$parser>::try_parse(result)
        }
    };
    ($req:expr, Json<$ve:ty>) => {
        $crate::send!($req, $crate::Json, $crate::JsonExtractor, $ve)
    };
    ($req:expr, $ve:ty) => {
        $crate::send!($req, $crate::Json, $crate::JsonExtractor, $ve)
    };
    ($req:expr, $parser:ty, $vet:ty, $ve:ty) => {
        async {
            use $vet;
            let result = $crate::__internal::send(
                $req,
                $crate::__internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None::<bool>,
                    <$ve>::require_headers(),
                ),
            )
            .await?;
            let result = <$parser>::try_parse::<$ve>(result)?;
            <$ve>::try_extract(result)
        }
    };
}

/// Internal macro
#[macro_export]
#[doc(hidden)]
macro_rules! _send_with {
    ($req:expr, $config:expr) => {
        $crate::_send_with!($req, $crate::Auto, (), $config)
    };
    ($req:expr, (), $config:expr) => {
        async {
            let _ = $crate::__internal::send($req, $config.merge($crate::_function_path!(), false))
                .await?;
            Ok(())
        }
    };
    ($req:expr, Body, $config:expr) => {
        async {
            $crate::__internal::send($req, $config.merge($crate::_function_path!(), true))
                .await
                .and_then(|c| c.try_into())
        }
    };
    ($req:expr, Json, $config:expr) => {
        $crate::_send_with!($req, $crate::Json, (), $config)
    };
    ($req:expr, Xml, $config:expr) => {
        $crate::_send_with!($req, $crate::Xml, (), $config)
    };
    ($req:expr, Text, $config:expr) => {
        $crate::_send_with!($req, $crate::Text, (), $config)
    };
    ($req:expr, $parser:ty, (), $config:expr) => {
        async {
            let result =
                $crate::__internal::send($req, $config.merge($crate::_function_path!(), false))
                    .await?;
            <$parser>::try_parse(result)
        }
    };
    ($req:expr, Json<$ve:ty>, $config:expr) => {
        $crate::_send_with!($req, $crate::Json, $crate::JsonExtractor, $ve, $config)
    };
    ($req:expr, $ve:ty, $config:expr) => {
        $crate::_send_with!($req, $crate::Json, $crate::JsonExtractor, $ve, $config)
    };
    ($req:expr, $parser:ty, $vet:ty, $ve:ty, $config:expr) => {
        async {
            use $vet;
            let result = $crate::__internal::send(
                $req,
                $config.merge($crate::_function_path!(), <$ve>::require_headers()),
            )
            .await?;
            let result = <$parser>::try_parse::<$ve>(result)?;
            <$ve>::try_extract(result)
        }
    };
}

/// Send the payload as JSON
///
/// # Forms
///
/// - `send_json!(req, json)` -> `impl Future<Output = ApiResult<T>>`
///     - send json, and parse response as json or xml based on response
/// - `send_json!(req, json, ())` -> `impl Future<Output = ApiResult<()>>`
///     - send json, verify response status, then discard response
/// - `send_json!(req, json, Body)` -> `impl Future<Output = ApiResult<apisdk::ResponseBody>>`
///     - send json, verify response status, and decode response body
/// - `send_json!(req, json, Json)` -> `impl Future<Output = ApiResult<T>>`
///     - send the request, parse response as json, then use serde_json to deserialize it
/// - `send_json!(req, json, Xml)` -> `impl Future<Output = ApiResult<T>>`
///     - send the request, parse response as xml, then use quick_xml to deserialize it
/// - `send_json!(req, json, Text)` -> `impl Future<Output = ApiResult<T>>`
///     - send the request, parse response as text, then use FromStr to deserialize it
/// - `send_json!(req, json, OtherType)` -> `impl Future<Output = ApiResult<T>>`
///     - send json, parse response as json, and use `OtherType` as JsonExtractor
/// - `send_json!(req, json, Json<OtherType>)` -> `impl Future<Output = ApiResult<T>>`
///     - send json, parse response as json, and use `OtherType` as JsonExtractor
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
        $crate::send_json!($req, $json, $crate::Auto, ())
    };
    ($req:expr, $json:expr, ()) => {
        async {
            let _ = $crate::__internal::send_json(
                $req,
                &($json),
                $crate::__internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None::<bool>,
                    false,
                ),
            )
            .await?;
            Ok(())
        }
    };
    ($req:expr, $json:expr, Body) => {
        async {
            $crate::__internal::send_json(
                $req,
                &($json),
                $crate::__internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None::<bool>,
                    true,
                ),
            )
            .await
            .and_then(|c| c.try_into())
        }
    };
    ($req:expr, $json:expr, Json) => {
        $crate::send_json!($req, $json, $crate::Json, ())
    };
    ($req:expr, $json:expr, Xml) => {
        $crate::send_json!($req, $json, $crate::Xml, ())
    };
    ($req:expr, $json:expr, Text) => {
        $crate::send_json!($req, $json, $crate::Text, ())
    };
    ($req:expr, $json:expr, $parser:ty, ()) => {
        async {
            let result = $crate::__internal::send_json(
                $req,
                &($json),
                $crate::__internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None::<bool>,
                    false,
                ),
            )
            .await?;
            <$parser>::try_parse(result)
        }
    };
    ($req:expr, $json:expr, Json<$ve:ty>) => {
        $crate::send_json!($req, $json, $crate::Json, $crate::JsonExtractor, $ve)
    };
    ($req:expr, $json:expr, $ve:ty) => {
        $crate::send_json!($req, $json, $crate::Json, $crate::JsonExtractor, $ve)
    };
    ($req:expr, $json:expr, $parser:ty, $vet:ty, $ve:ty) => {
        async {
            use $vet;
            let result = $crate::__internal::send_json(
                $req,
                &($json),
                $crate::__internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None::<bool>,
                    <$ve>::require_headers(),
                ),
            )
            .await?;
            let result = <$parser>::try_parse::<$ve>(result)?;
            <$ve>::try_extract(result)
        }
    };
}

/// Internal macro
#[macro_export]
#[doc(hidden)]
macro_rules! _send_json_with {
    ($req:expr, $json:expr, $config:expr) => {
        $crate::_send_json_with!($req, $json, $crate::Auto, (), $config)
    };
    ($req:expr, $json:expr, (), $config:expr) => {
        async {
            let _ = $crate::__internal::send_json(
                $req,
                &($json),
                $config.merge($crate::_function_path!(), false),
            )
            .await?;
            Ok(())
        }
    };
    ($req:expr, $json:expr, Body, $config:expr) => {
        async {
            $crate::__internal::send_json(
                $req,
                &($json),
                $config.merge($crate::_function_path!(), true),
            )
            .await
            .and_then(|c| c.try_into())
        }
    };
    ($req:expr, $json:expr, Json, $config:expr) => {
        $crate::_send_json_with!($req, $json, $crate::Json, (), $config)
    };
    ($req:expr, $json:expr, Xml, $config:expr) => {
        $crate::_send_json_with!($req, $json, $crate::Xml, (), $config)
    };
    ($req:expr, $json:expr, Text, $config:expr) => {
        $crate::_send_json_with!($req, $json, $crate::Text, (), $config)
    };
    ($req:expr, $json:expr, $parser:ty, (), $config:expr) => {
        async {
            let result = $crate::__internal::send_json(
                $req,
                &($json),
                $config.merge($crate::_function_path!(), false),
            )
            .await?;
            <$parser>::try_parse(result)
        }
    };
    ($req:expr, $json:expr, Json<$ve:ty>, $config:expr) => {
        $crate::_send_json_with!(
            $req,
            $json,
            $crate::Json,
            $crate::JsonExtractor,
            $ve,
            $config
        )
    };
    ($req:expr, $json:expr, $ve:ty, $config:expr) => {
        $crate::_send_json_with!(
            $req,
            $json,
            $crate::Json,
            $crate::JsonExtractor,
            $ve,
            $config
        )
    };
    ($req:expr, $json:expr, $parser:ty, $vet:ty, $ve:ty, $config:expr) => {
        async {
            use $vet;
            let result = $crate::__internal::send_json(
                $req,
                &($json),
                $config.merge($crate::_function_path!(), <$ve>::require_headers()),
            )
            .await?;
            let result = <$parser>::try_parse::<$ve>(result)?;
            <$ve>::try_extract(result)
        }
    };
}

/// Send the payload as XML, which will be serialized by quick_xml
///
/// # Forms
///
/// - `send_xml!(req, xml)` -> `impl Future<Output = ApiResult<T>>`
///     - send xml, and parse response as json or xml based on response
/// - `send_xml!(req, xml, ())` -> `impl Future<Output = ApiResult<()>>`
///     - send xml, verify response status, then discard response
/// - `send_xml!(req, xml, Body)` -> `impl Future<Output = ApiResult<apisdk::ResponseBody>>`
///     - send xml, verify response status, and decode response body
/// - `send_xml!(req, xml, Json)` -> `impl Future<Output = ApiResult<T>>`
///     - send the request, parse response as json, then use serde_json to deserialize it
/// - `send_xml!(req, xml, Xml)` -> `impl Future<Output = ApiResult<T>>`
///     - send the request, parse response as xml, then use quick_xml to deserialize it
/// - `send_xml!(req, xml, Text)` -> `impl Future<Output = ApiResult<T>>`
///     - send the request, parse response as text, then use FromStr to deserialize it
/// - `send_xml!(req, xml, OtherType)` -> `impl Future<Output = ApiResult<T>>`
///     - send xml, parse response as json, and use `OtherType` as JsonExtractor
/// - `send_xml!(req, xml, Json<OtherType>)` -> `impl Future<Output = ApiResult<T>>`
///     - send xml, parse response as json, and use `OtherType` as JsonExtractor
///
/// # Examples
///
/// ```
/// #[derive(serde::Serialize)]
/// struct Data {
///     key: String,
/// }
///
/// let data = Data { key: "value".to_string() };
/// let req = client.post("/path/api").await?;
/// let res: TypeOfResponse = send_xml!(req, data).await?;
/// ```
///
/// Please reference `send` for more information
#[macro_export]
macro_rules! send_xml {
    ($req:expr, $xml:expr) => {
        $crate::send_xml!($req, $xml, $crate::Auto, ())
    };
    ($req:expr, $xml:expr, ()) => {
        async {
            let _ = $crate::__internal::send_xml(
                $req,
                &($xml),
                $crate::__internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None::<bool>,
                    false,
                ),
            )
            .await?;
            Ok(())
        }
    };
    ($req:expr, $xml:expr, Body) => {
        async {
            $crate::__internal::send_xml(
                $req,
                &($xml),
                $crate::__internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None::<bool>,
                    true,
                ),
            )
            .await
            .and_then(|c| c.try_into())
        }
    };
    ($req:expr, $xml:expr, Json) => {
        $crate::send_xml!($req, $xml, $crate::Json, ())
    };
    ($req:expr, $xml:expr, Xml) => {
        $crate::send_xml!($req, $xml, $crate::Xml, ())
    };
    ($req:expr, $xml:expr, Text) => {
        $crate::send_xml!($req, $xml, $crate::Text, ())
    };
    ($req:expr, $xml:expr, $parser:ty, ()) => {
        async {
            let result = $crate::__internal::send_xml(
                $req,
                &($xml),
                $crate::__internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None::<bool>,
                    false,
                ),
            )
            .await?;
            <$parser>::try_parse(result)
        }
    };
    ($req:expr, $xml:expr, Json<$ve:ty>) => {
        $crate::send_xml!($req, $xml, $crate::Json, $crate::JsonExtractor, $ve)
    };
    ($req:expr, $xml:expr, $ve:ty) => {
        $crate::send_xml!($req, $xml, $crate::Json, $crate::JsonExtractor, $ve)
    };
    ($req:expr, $xml:expr, $parser:ty, $vet:ty, $ve:ty) => {
        async {
            use $vet;
            let result = $crate::__internal::send_xml(
                $req,
                &($xml),
                $crate::__internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None::<bool>,
                    <$ve>::require_headers(),
                ),
            )
            .await?;
            let result = <$parser>::try_parse::<$ve>(result)?;
            <$ve>::try_extract(result)
        }
    };
}

/// Internal macro
#[macro_export]
#[doc(hidden)]
macro_rules! _send_xml_with {
    ($req:expr, $xml:expr, $config:expr) => {
        $crate::_send_xml_with!($req, $xml, $crate::Auto, (), $config)
    };
    ($req:expr, $xml:expr, (), $config:expr) => {
        async {
            let _ = $crate::__internal::send_xml(
                $req,
                &($xml),
                $config.merge($crate::_function_path!(), false),
            )
            .await?;
            Ok(())
        }
    };
    ($req:expr, $xml:expr, Body, $config:expr) => {
        async {
            $crate::__internal::send_xml(
                $req,
                &($xml),
                $config.merge($crate::_function_path!(), true),
            )
            .await
            .and_then(|c| c.try_into())
        }
    };
    ($req:expr, $xml:expr, Json, $config:expr) => {
        $crate::_send_xml_with!($req, $xml, $crate::Json, (), $config)
    };
    ($req:expr, $xml:expr, Xml, $config:expr) => {
        $crate::_send_xml_with!($req, $xml, $crate::Xml, (), $config)
    };
    ($req:expr, $xml:expr, Text, $config:expr) => {
        $crate::_send_xml_with!($req, $xml, $crate::Text, (), $config)
    };
    ($req:expr, $xml:expr, $parser:ty, (), $config:expr) => {
        async {
            let result = $crate::__internal::send_xml(
                $req,
                &($xml),
                $config.merge($crate::_function_path!(), false),
            )
            .await?;
            <$parser>::try_parse(result)
        }
    };
    ($req:expr, $xml:expr, Json<$ve:ty>, $config:expr) => {
        $crate::_send_xml_with!(
            $req,
            $xml,
            $crate::Json,
            $crate::JsonExtractor,
            $ve,
            $config
        )
    };
    ($req:expr, $xml:expr, $ve:ty, $config:expr) => {
        $crate::_send_xml_with!(
            $req,
            $xml,
            $crate::Json,
            $crate::JsonExtractor,
            $ve,
            $config
        )
    };
    ($req:expr, $xml:expr, $parser:ty, $vet:ty, $ve:ty, $config:expr) => {
        async {
            use $vet;
            let result = $crate::__internal::send_xml(
                $req,
                &($xml),
                $config.merge($crate::_function_path!(), <$ve>::require_headers()),
            )
            .await?;
            let result = <$parser>::try_parse::<$ve>(result)?;
            <$ve>::try_extract(result)
        }
    };
}

/// Send the payload as form
///
/// # Forms
///
/// - `send_form!(req, form)` -> `impl Future<Output = ApiResult<T>>`
///     - send form, and parse response as json or xml based on response
/// - `send_form!(req, form, ())` -> `impl Future<Output = ApiResult<()>>`
///     - send form, verify response status, then discard response
/// - `send_form!(req, form, Body)` -> `impl Future<Output = ApiResult<apisdk::ResponseBody>>`
///     - send form, verify response status, and decode response body
/// - `send_form!(req, form, Json)` -> `impl Future<Output = ApiResult<T>>`
///     - send the request, parse response as json, then use serde_json to deserialize it
/// - `send_form!(req, form, Xml)` -> `impl Future<Output = ApiResult<T>>`
///     - send the request, parse response as xml, then use quick_xml to deserialize it
/// - `send_form!(req, form, Text)`-> `impl Future<Output = ApiResult<T>>`
///     - send the request, parse response as text, then use FromStr to deserialize it
/// - `send_form!(req, form, OtherType)` -> `impl Future<Output = ApiResult<T>>`
///     - send form, parse response as json, and use `OtherType` as JsonExtractor
/// - `send_form!(req, form, Json<OtherType>)` -> `impl Future<Output = ApiResult<T>>`
///     - send form, parse response as json, and use `OtherType` as JsonExtractor
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
        $crate::send_form!($req, $form, $crate::Auto, ())
    };
    ($req:expr, $form:expr, ()) => {
        async {
            let _ = $crate::__internal::send_form(
                $req,
                $form,
                $crate::__internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None::<bool>,
                    false,
                ),
            )
            .await?;
            Ok(())
        }
    };
    ($req:expr, $form:expr, Body) => {
        async {
            $crate::__internal::send_form(
                $req,
                $form,
                $crate::__internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None::<bool>,
                    true,
                ),
            )
            .await
            .and_then(|c| c.try_into())
        }
    };
    ($req:expr, $form:expr, Json) => {
        $crate::send_form!($req, $form, $crate::Json, ())
    };
    ($req:expr, $form:expr, Xml) => {
        $crate::send_form!($req, $form, $crate::Xml, ())
    };
    ($req:expr, $form:expr, Text) => {
        $crate::send_form!($req, $form, $crate::Text, ())
    };
    ($req:expr, $form:expr, $parser:ty, ()) => {
        async {
            let result = $crate::__internal::send_form(
                $req,
                $form,
                $crate::__internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None::<bool>,
                    false,
                ),
            )
            .await?;
            <$parser>::try_parse(result)
        }
    };
    ($req:expr, $form:expr, Json<$ve:ty>) => {
        $crate::send_form!($req, $form, $crate::Json, $crate::JsonExtractor, $ve)
    };
    ($req:expr, $form:expr, $ve:ty) => {
        $crate::send_form!($req, $form, $crate::Json, $crate::JsonExtractor, $ve)
    };
    ($req:expr, $form:expr, $parser:ty, $vet:ty, $ve:ty) => {
        async {
            use $vet;
            let result = $crate::__internal::send_form(
                $req,
                $form,
                $crate::__internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None::<bool>,
                    <$ve>::require_headers(),
                ),
            )
            .await?;
            let result = <$parser>::try_parse::<$ve>(result)?;
            <$ve>::try_extract(result)
        }
    };
}

/// Internal macro
#[macro_export]
#[doc(hidden)]
macro_rules! _send_form_with {
    ($req:expr, $form:expr, $config:expr) => {
        $crate::_send_form_with!($req, $form, $crate::Auto, (), $config)
    };
    ($req:expr, $form:expr, (), $config:expr) => {
        async {
            let _ = $crate::__internal::send_form(
                $req,
                $form,
                $config.merge($crate::_function_path!(), false),
            )
            .await?;
            Ok(())
        }
    };
    ($req:expr, $form:expr, Body, $config:expr) => {
        async {
            $crate::__internal::send_form(
                $req,
                $form,
                $config.merge($crate::_function_path!(), true),
            )
            .await
            .and_then(|c| c.try_into())
        }
    };
    ($req:expr, $form:expr, Json, $config:expr) => {
        $crate::_send_form_with!($req, $form, $crate::Json, (), $config)
    };
    ($req:expr, $form:expr, Xml, $config:expr) => {
        $crate::_send_form_with!($req, $form, $crate::Xml, (), $config)
    };
    ($req:expr, $form:expr, Text, $config:expr) => {
        $crate::_send_form_with!($req, $form, $crate::Text, (), $config)
    };
    ($req:expr, $form:expr, $parser:ty, (), $config:expr) => {
        async {
            let result = $crate::__internal::send_form(
                $req,
                $form,
                $config.merge($crate::_function_path!(), false),
            )
            .await?;
            <$parser>::try_parse(result)
        }
    };
    ($req:expr, $form:expr, Json<$ve:ty>, $config:expr) => {
        $crate::_send_form_with!(
            $req,
            $form,
            $crate::Json,
            $crate::JsonExtractor,
            $ve,
            $config
        )
    };
    ($req:expr, $form:expr, $ve:ty, $config:expr) => {
        $crate::_send_form_with!(
            $req,
            $form,
            $crate::Json,
            $crate::JsonExtractor,
            $ve,
            $config
        )
    };
    ($req:expr, $form:expr, $parser:ty, $vet:ty, $ve:ty, $config:expr) => {
        async {
            use $vet;
            let result = $crate::__internal::send_form(
                $req,
                $form,
                $config.merge($crate::_function_path!(), <$ve>::require_headers()),
            )
            .await?;
            let result = <$parser>::try_parse::<$ve>(result)?;
            <$ve>::try_extract(result)
        }
    };
}

/// Send the payload as multipart form
///
/// # Forms
///
/// - `send_multipart!(req, form)` -> `impl Future<Output = ApiResult<T>>`
///     - send form, and parse response as json or xml based on response
/// - `send_multipart!(req, form, ())` -> `impl Future<Output = ApiResult<()>>`
///     - send form, verify response status, then discard response
/// - `send_multipart!(req, form, Body)` -> `impl Future<Output = ApiResult<apisdk::ResponseBody>>`
///     - send form, verify response status, and decode response body
/// - `send_multipart!(req, form, Json)` -> `impl Future<Output = ApiResult<T>>`
///     - send the request, parse response as json, then use serde_json to deserialize it
/// - `send_multipart!(req, form, Xml)` -> `impl Future<Output = ApiResult<T>>`
///     - send the request, parse response as xml, then use quick_xml to deserialize it
/// - `send_multipart!(req, form, Text)` -> `impl Future<Output = ApiResult<T>>`
///     - send the request, parse response as text, then use FromStr to deserialize it
/// - `send_multipart!(req, form, OtherType)` -> `impl Future<Output = ApiResult<T>>`
///     - send form, parse response as json, and use `OtherType` as JsonExtractor
/// - `send_multipart!(req, form, Json<OtherType>)` -> `impl Future<Output = ApiResult<T>>`
///     - send form, parse response as json, and use `OtherType` as JsonExtractor
///
/// # Examples
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
        $crate::send_multipart!($req, $form, $crate::Auto, ())
    };
    ($req:expr, $form:expr, ()) => {
        async {
            let _ = $crate::__internal::send_multipart(
                $req,
                $form,
                $crate::__internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None::<bool>,
                    false,
                ),
            )
            .await?;
            Ok(())
        }
    };
    ($req:expr, $form:expr, Body) => {
        async {
            $crate::__internal::send_multipart(
                $req,
                $form,
                $crate::__internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None::<bool>,
                    true,
                ),
            )
            .await
            .and_then(|c| c.try_into())
        }
    };
    ($req:expr, $form:expr, Json) => {
        $crate::send_multipart!($req, $form, $crate::Json, ())
    };
    ($req:expr, $form:expr, Xml) => {
        $crate::send_multipart!($req, $form, $crate::Xml, ())
    };
    ($req:expr, $form:expr, Text) => {
        $crate::send_multipart!($req, $form, $crate::Text, ())
    };
    ($req:expr, $form:expr, $parser:ty, ()) => {
        async {
            let result = $crate::__internal::send_multipart(
                $req,
                $form,
                $crate::__internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None::<bool>,
                    false,
                ),
            )
            .await?;
            <$parser>::try_parse(result)
        }
    };
    ($req:expr, $form:expr, Json<$ve:ty>) => {
        $crate::send_multipart!($req, $form, $crate::Json, $crate::JsonExtractor, $ve)
    };
    ($req:expr, $form:expr, $ve:ty) => {
        $crate::send_multipart!($req, $form, $crate::Json, $crate::JsonExtractor, $ve)
    };
    ($req:expr, $form:expr, $parser:ty, $vet:ty, $ve:ty) => {
        async {
            use $vet;
            let result = $crate::__internal::send_multipart(
                $req,
                $form,
                $crate::__internal::RequestConfigurator::new(
                    $crate::_function_path!(),
                    None::<bool>,
                    <$ve>::require_headers(),
                ),
            )
            .await?;
            let result = <$parser>::try_parse::<$ve>(result)?;
            <$ve>::try_extract(result)
        }
    };
}

/// Internal macro
#[macro_export]
#[doc(hidden)]
macro_rules! _send_multipart_with {
    ($req:expr, $form:expr, $config:expr) => {
        $crate::_send_multipart_with!($req, $form, $crate::Auto, (), $config)
    };
    ($req:expr, $form:expr, (), $config:expr) => {
        async {
            let _ = $crate::__internal::send_multipart(
                $req,
                $form,
                $config.merge($crate::_function_path!(), false),
            )
            .await?;
            Ok(())
        }
    };
    ($req:expr, $form:expr, Body, $config:expr) => {
        async {
            $crate::__internal::send_multipart(
                $req,
                $form,
                $config.merge($crate::_function_path!(), true),
            )
            .await
            .and_then(|c| c.try_into())
        }
    };
    ($req:expr, $form:expr, Json, $config:expr) => {
        $crate::_send_multipart_with!($req, $form, $crate::Json, (), $config)
    };
    ($req:expr, $form:expr, Xml, $config:expr) => {
        $crate::_send_multipart_with!($req, $form, $crate::Xml, (), $config)
    };
    ($req:expr, $form:expr, Text, $config:expr) => {
        $crate::_send_multipart_with!($req, $form, $crate::Text, (), $config)
    };
    ($req:expr, $form:expr, $parser:ty, (), $config:expr) => {
        async {
            let result = $crate::__internal::send_multipart(
                $req,
                $form,
                $config.merge($crate::_function_path!(), false),
            )
            .await?;
            <$parser>::try_parse(result)
        }
    };
    ($req:expr, $form:expr, Json<$ve:ty>, $config:expr) => {
        $crate::_send_multipart_with!(
            $req,
            $form,
            $crate::Json,
            $crate::JsonExtractor,
            $ve,
            $config
        )
    };
    ($req:expr, $form:expr, $ve:ty, $config:expr) => {
        $crate::_send_multipart_with!(
            $req,
            $form,
            $crate::Json,
            $crate::JsonExtractor,
            $ve,
            $config
        )
    };
    ($req:expr, $form:expr, $parser:ty, $vet:ty, $ve:ty, $config:expr) => {
        async {
            use $vet;
            let result = $crate::__internal::send_multipart(
                $req,
                $form,
                $config.merge($crate::_function_path!(), <$ve>::require_headers()),
            )
            .await?;
            let result = <$parser>::try_parse::<$ve>(result)?;
            <$ve>::try_extract(result)
        }
    };
}

/// Send and get raw response
///
/// # Forms
///
/// - `send_raw!(req)`
///     - send request, and return raw response
#[macro_export]
macro_rules! send_raw {
    ($req:expr) => {
        $crate::__internal::send_raw(
            $req,
            $crate::__internal::RequestConfigurator::new(
                $crate::_function_path!(),
                None::<bool>,
                false,
            ),
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
