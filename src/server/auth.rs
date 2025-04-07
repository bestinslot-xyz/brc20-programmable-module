use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use jsonrpsee::server::middleware::rpc::layer::ResponseFuture;
use jsonrpsee::server::middleware::rpc::RpcServiceT;
use jsonrpsee::types::{ErrorObject, Id, Request};
use jsonrpsee::MethodResponse;
use tower_http::validate_request::ValidateRequest;

#[derive(Debug, Clone)]
/// A marker type for authorized requests.
pub struct Authorized {}

impl Default for Authorized {
    fn default() -> Self {
        Self {}
    }
}

/// A middleware that checks for HTTP Basic Authentication.
/// It uses the `rpc_username` and `rpc_password` to create a Basic Auth header.
/// If the request contains the correct Basic Auth header, it adds an `Authorized`
/// extension to the request.
///
/// It never rejects requests, it just adds the `Authorized` extension if the header is present.
#[derive(Clone)]
pub struct HttpNonBlockingAuth {
    header: Option<String>,
    allow_all: bool,
}

impl HttpNonBlockingAuth {
    pub fn allow() -> Self {
        Self {
            header: None,
            allow_all: true,
        }
    }

    pub fn new(rpc_username: &String, rpc_password: &String) -> Self {
        Self {
            header: Some(format!(
                "Basic {}",
                BASE64_STANDARD
                    .encode(format!("{}:{}", rpc_username, rpc_password).as_bytes())
                    .to_string()
            )),
            allow_all: false,
        }
    }
}

impl<B> ValidateRequest<B> for HttpNonBlockingAuth {
    type ResponseBody = B;

    fn validate(
        &mut self,
        request: &mut hyper::Request<B>,
    ) -> Result<(), hyper::Response<Self::ResponseBody>> {
        if self.allow_all || request
            .headers()
            .get("Authorization")
            .and_then(|header| header.to_str().ok())
            == self.header.as_deref()
        {
            request.extensions_mut().insert(Authorized::default());
        }

        return Ok(());
    }
}

/// A middleware that allows all requests to the `eth_*` methods.
/// It checks if the request is authorized.
/// If the request is not authorized and the method does not start with `eth_`,
/// it returns an error.
pub struct RpcAuthAllowEth<S> {
    service: S,
}

impl<S> RpcAuthAllowEth<S> {
    pub fn new(service: S) -> Self {
        Self { service }
    }

    fn validate(&self, authorized: bool, method: &str) -> Result<(), ()> {
        if !authorized {
            let is_eth_method = method.starts_with("eth_");
            if !is_eth_method {
                return Err(());
            }
        }
        Ok(())
    }
}

impl<'a, S> RpcServiceT<'a> for RpcAuthAllowEth<S>
where
    S: RpcServiceT<'a> + Send + Sync,
{
    type Future = ResponseFuture<S::Future>;

    fn call(&self, req: Request<'a>) -> Self::Future {
        if self
            .validate(
                req.extensions().get::<Authorized>().is_some(),
                req.method_name(),
            )
            .is_err()
        {
            return ResponseFuture::ready(MethodResponse::error(
                Id::Number(401),
                ErrorObject::borrowed(401, "Unauthorized", None),
            ));
        }

        ResponseFuture::future(self.service.call(req))
    }
}

#[cfg(test)]
mod tests {
    use jsonrpsee::server::HttpBody;

    use super::*;

    struct MockRpcService;

    impl RpcServiceT<'_> for MockRpcService {
        type Future = std::future::Ready<MethodResponse>;

        fn call(&self, _: Request<'_>) -> Self::Future {
            std::future::ready(MethodResponse::response(
                Id::Number(1),
                jsonrpsee::ResponsePayload::success("success"),
                usize::MAX,
            ))
        }
    }

    #[tokio::test]
    async fn test_rpc_auth_unauthorized_eth_success() {
        let mut auth = HttpNonBlockingAuth::new(&"user".to_string(), &"password".to_string());
        let validator = RpcAuthAllowEth::new(MockRpcService);
        let mut request = hyper::Request::builder()
            .header("Authorization", "Basic asdfgh==")
            .body(HttpBody::empty())
            .unwrap();

        assert!(auth.validate(&mut request).is_ok());

        let mut rpc_request =
            jsonrpsee::types::Request::new("eth_blockNumber".into(), None, Id::Number(1));

        rpc_request.extensions = request.extensions().clone();

        assert!(validator.call(rpc_request).await.is_success());
    }

    #[tokio::test]
    async fn test_rpc_auth_authorized_eth_success() {
        let mut auth = HttpNonBlockingAuth::new(&"user".to_string(), &"password".to_string());
        let validator = RpcAuthAllowEth::new(MockRpcService);
        let mut request = hyper::Request::builder()
            .header("Authorization", "Basic dXNlcjpwYXNzd29yZA==")
            .body(HttpBody::empty())
            .unwrap();

        assert!(auth.validate(&mut request).is_ok());
        assert!(request.extensions().get::<Authorized>().is_some());

        let mut rpc_request =
            jsonrpsee::types::Request::new("eth_blockNumber".into(), None, Id::Number(1));

        rpc_request.extensions = request.extensions().clone();

        assert!(validator.call(rpc_request).await.is_success());
    }

    #[tokio::test]
    async fn test_rpc_auth_authorized_brc20_success() {
        let mut auth = HttpNonBlockingAuth::new(&"user".to_string(), &"password".to_string());
        let validator = RpcAuthAllowEth::new(MockRpcService);
        let mut request = hyper::Request::builder()
            .header("Authorization", "Basic dXNlcjpwYXNzd29yZA==")
            .body(HttpBody::empty())
            .unwrap();

        assert!(auth.validate(&mut request).is_ok());
        assert!(request.extensions().get::<Authorized>().is_some());

        let mut rpc_request =
            jsonrpsee::types::Request::new("brc20_hello".into(), None, Id::Number(1));

        rpc_request.extensions = request.extensions().clone();

        assert!(validator.call(rpc_request).await.is_success());
    }

    #[tokio::test]
    async fn test_rpc_auth_wrong_credentials_brc20_error() {
        let mut auth = HttpNonBlockingAuth::new(&"user".to_string(), &"password".to_string());
        let validator = RpcAuthAllowEth::new(MockRpcService);
        let mut request = hyper::Request::builder()
            .header("Authorization", "Basic asdfgh==")
            .body(HttpBody::empty())
            .unwrap();

        assert!(auth.validate(&mut request).is_ok());

        let mut rpc_request =
            jsonrpsee::types::Request::new("brc20_hello".into(), None, Id::Number(1));

        rpc_request.extensions = request.extensions().clone();

        assert!(validator.call(rpc_request).await.is_error());
    }

    #[tokio::test]
    async fn test_rpc_auth_no_header_brc20_error() {
        let mut auth = HttpNonBlockingAuth::new(&"user".to_string(), &"password".to_string());
        let validator = RpcAuthAllowEth::new(MockRpcService);
        let mut request = hyper::Request::builder().body(HttpBody::empty()).unwrap();

        assert!(auth.validate(&mut request).is_ok());

        let mut rpc_request =
            jsonrpsee::types::Request::new("brc20_hello".into(), None, Id::Number(1));

        rpc_request.extensions = request.extensions().clone();

        assert!(validator.call(rpc_request).await.is_error());
    }

    #[tokio::test]
    async fn test_rpc_auth_no_header_eth_success() {
        let mut auth = HttpNonBlockingAuth::new(&"user".to_string(), &"password".to_string());
        let validator = RpcAuthAllowEth::new(MockRpcService);
        let mut request = hyper::Request::builder().body(HttpBody::empty()).unwrap();

        assert!(auth.validate(&mut request).is_ok());

        let mut rpc_request = jsonrpsee::types::Request::new("eth_yo".into(), None, Id::Number(1));

        rpc_request.extensions = request.extensions().clone();

        assert!(validator.call(rpc_request).await.is_success());
    }
}
