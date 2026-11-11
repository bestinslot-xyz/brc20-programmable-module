use std::collections::HashSet;
use std::future::Future;
use std::sync::Arc;

use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use jsonrpsee::core::middleware::{Batch, BatchEntry, BatchEntryErr, Notification, ResponseFuture};
use jsonrpsee::server::middleware::rpc::RpcServiceT;
use jsonrpsee::types::{ErrorObject, Id, Request};
use jsonrpsee::MethodResponse;
use subtle::ConstantTimeEq;
use tower_http::validate_request::ValidateRequest;

use crate::global::validate_rpc_credentials;

#[derive(Debug, Clone)]
/// A marker type for authorized requests.
pub struct Authorized {}

impl Default for Authorized {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Clone)]
enum AuthMode {
    /// Requests carrying this exact `Authorization` header value are authorized.
    Basic(Arc<[u8]>),
    /// No request is authorized.
    DenyAll,
}

/// A middleware that marks requests as authorized by adding an `Authorized` extension.
///
/// It authorizes either requests with the correct HTTP Basic Auth header, or no request at all.
/// Basic mode only accepts credentials that pass [`validate_rpc_credentials`].
///
/// It never rejects requests, it just adds the `Authorized` extension when the mode allows it.
/// It deliberately has no `Debug` implementation, so the expected header never reaches a log.
#[derive(Clone)]
pub struct HttpNonBlockingAuth {
    mode: AuthMode,
}

impl HttpNonBlockingAuth {
    /// Authorizes no request.
    pub fn deny() -> Self {
        Self {
            mode: AuthMode::DenyAll,
        }
    }

    /// Authorizes requests carrying HTTP Basic Auth with `rpc_username` and `rpc_password`.
    ///
    /// Fails when the credentials do not pass [`validate_rpc_credentials`].
    pub fn basic(rpc_username: &str, rpc_password: &str) -> Result<Self, String> {
        validate_rpc_credentials(rpc_username, rpc_password)?;
        let expected = format!(
            "Basic {}",
            BASE64_STANDARD.encode(format!("{}:{}", rpc_username, rpc_password).as_bytes())
        );
        Ok(Self {
            mode: AuthMode::Basic(expected.into_bytes().into()),
        })
    }

    /// Whether this middleware can authorize any request, i.e. whether it guards the indexer
    /// methods with credentials rather than rejecting them outright.
    pub fn requires_credentials(&self) -> bool {
        matches!(self.mode, AuthMode::Basic(_))
    }

    fn is_authorized<B>(&self, request: &hyper::Request<B>) -> bool {
        match &self.mode {
            AuthMode::DenyAll => false,
            AuthMode::Basic(expected) => {
                let Some(provided) = request
                    .headers()
                    .get(hyper::header::AUTHORIZATION)
                    .and_then(|header| header.to_str().ok())
                else {
                    return false;
                };
                // A length mismatch is unequal; equal-length values compare in constant time.
                provided.as_bytes().ct_eq(expected).into()
            }
        }
    }
}

impl<B> ValidateRequest<B> for HttpNonBlockingAuth {
    type ResponseBody = B;

    fn validate(
        &mut self,
        request: &mut hyper::Request<B>,
    ) -> Result<(), hyper::Response<Self::ResponseBody>> {
        if self.is_authorized(request) {
            request.extensions_mut().insert(Authorized::default());
        }

        return Ok(());
    }
}

/// A middleware that denies requests to methods in the denylist unless they are authorised.
/// It checks if the request has the `Authorized` extension.
/// If the request is not authorized and the method is in the denylist, it returns an error.
/// Otherwise, it calls the inner service.
/// It is used to protect sensitive methods from unauthorized access.
/// It is used in conjunction with the `HttpNonBlockingAuth` middleware.
#[derive(Clone)]
pub struct RpcAuthMiddleware<S> {
    service: S,
    denylist: HashSet<String>,
}

impl<S> RpcAuthMiddleware<S> {
    pub fn new<I: IntoIterator<Item = String> + Clone>(service: S, denylist: &I) -> Self {
        Self {
            service,
            denylist: denylist.clone().into_iter().collect(),
        }
    }

    fn validate_call(&self, request: &Request<'_>) -> bool {
        request.extensions().get::<Authorized>().is_some()
            || !self.denylist.contains(request.method_name())
    }

    fn validate_notification(&self, notification: &Notification<'_>) -> bool {
        notification.extensions().get::<Authorized>().is_some()
            || !self.denylist.contains(notification.method_name())
    }
}

impl<S> RpcServiceT for RpcAuthMiddleware<S>
where
    S: RpcServiceT<
        MethodResponse = MethodResponse,
        BatchResponse = MethodResponse,
        NotificationResponse = MethodResponse,
    >,
{
    type MethodResponse = S::MethodResponse;
    type NotificationResponse = S::NotificationResponse;
    type BatchResponse = S::BatchResponse;

    fn call<'a>(&self, req: Request<'a>) -> impl Future<Output = Self::MethodResponse> + Send + 'a {
        if !self.validate_call(&req) {
            ResponseFuture::ready(MethodResponse::error(
                req.id(),
                ErrorObject::borrowed(401, "Unauthorized", None),
            ))
        } else {
            ResponseFuture::future(self.service.call(req))
        }
    }

    fn notification<'a>(
        &self,
        notification: Notification<'a>,
    ) -> impl Future<Output = Self::NotificationResponse> + Send + 'a {
        if !self.validate_notification(&notification) {
            // Notifications are not expected to return a response
            ResponseFuture::ready(MethodResponse::notification())
        } else {
            ResponseFuture::future(self.service.notification(notification))
        }
    }

    fn batch<'a>(
        &self,
        mut batch: Batch<'a>,
    ) -> impl Future<Output = Self::BatchResponse> + Send + 'a {
        for entry in batch.iter_mut() {
            match entry {
                Ok(BatchEntry::Call(req)) => {
                    if !self.validate_call(&req) {
                        *entry = Err(BatchEntryErr::new(
                            req.id(),
                            ErrorObject::borrowed(401, "Unauthorized", None),
                        ));
                    }
                }
                Ok(BatchEntry::Notification(notification)) => {
                    if !self.validate_notification(&notification) {
                        *entry = Err(BatchEntryErr::new(
                            Id::Number(0),
                            ErrorObject::borrowed(401, "Unauthorized", None),
                        ));
                    }
                }
                Err(_) => {
                    // Ignore errors
                }
            }
        }
        self.service.batch(batch)
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use jsonrpsee::core::middleware::{Batch, Notification};
    use jsonrpsee::server::HttpBody;
    use jsonrpsee::types::Id;
    use jsonrpsee::ResponsePayload;

    use super::*;

    const TEST_USER: &str = "indexer";
    const TEST_PASSWORD: &str = "3f9c2a7e5b1d48c6a0e7f2b9d4c1a8e6";

    fn test_auth() -> HttpNonBlockingAuth {
        HttpNonBlockingAuth::basic(TEST_USER, TEST_PASSWORD).unwrap()
    }

    fn test_header() -> String {
        format!(
            "Basic {}",
            BASE64_STANDARD.encode(format!("{}:{}", TEST_USER, TEST_PASSWORD))
        )
    }

    fn authorizes(auth: &HttpNonBlockingAuth, header: Option<&[u8]>) -> bool {
        let mut builder = hyper::Request::builder();
        if let Some(header) = header {
            builder = builder.header(
                hyper::header::AUTHORIZATION,
                hyper::header::HeaderValue::from_bytes(header).unwrap(),
            );
        }
        let mut request = builder.body(HttpBody::empty()).unwrap();
        assert!(auth.clone().validate(&mut request).is_ok());
        request.extensions().get::<Authorized>().is_some()
    }

    #[test]
    fn test_basic_rejects_weak_credentials() {
        assert!(HttpNonBlockingAuth::basic("user", "password").is_err());
        assert!(HttpNonBlockingAuth::basic("", TEST_PASSWORD).is_err());
        assert!(HttpNonBlockingAuth::basic(TEST_USER, "").is_err());
        assert!(HttpNonBlockingAuth::basic(TEST_USER, TEST_PASSWORD).is_ok());
    }

    #[test]
    fn test_requires_credentials() {
        assert!(test_auth().requires_credentials());
        assert!(!HttpNonBlockingAuth::deny().requires_credentials());
    }

    #[test]
    fn test_basic_header_comparison() {
        let auth = test_auth();
        let header = test_header();

        assert!(authorizes(&auth, Some(header.as_bytes())));
        // Missing header
        assert!(!authorizes(&auth, None));
        // Empty header
        assert!(!authorizes(&auth, Some(b"")));
        // Prefix and extension of the expected value (length mismatch)
        assert!(!authorizes(
            &auth,
            Some(&header.as_bytes()[..header.len() - 1])
        ));
        assert!(!authorizes(&auth, Some(format!("{header}=").as_bytes())));
        // Same length, one byte different
        let mut flipped = header.clone().into_bytes();
        let last = flipped.len() - 1;
        flipped[last] = if flipped[last] == b'A' { b'B' } else { b'A' };
        assert!(!authorizes(&auth, Some(&flipped)));
        // Non-UTF-8 header value
        let mut non_utf8 = header.clone().into_bytes();
        non_utf8[last] = 0xff;
        assert!(!authorizes(&auth, Some(&non_utf8)));
        // Wrong password, wrong scheme
        let wrong = format!(
            "Basic {}",
            BASE64_STANDARD.encode(format!("{}:{}x", TEST_USER, TEST_PASSWORD))
        );
        assert!(!authorizes(&auth, Some(wrong.as_bytes())));
        let bearer = header.replacen("Basic", "Bearer", 1);
        assert!(!authorizes(&auth, Some(bearer.as_bytes())));
    }

    pub struct MockRpcService;

    impl RpcServiceT for MockRpcService {
        type MethodResponse = MethodResponse;
        type NotificationResponse = MethodResponse;
        type BatchResponse = MethodResponse;

        fn call<'a>(
            &self,
            req: Request<'a>,
        ) -> impl Future<Output = Self::MethodResponse> + Send + 'a {
            std::future::ready(MethodResponse::response(
                req.id(),
                ResponsePayload::success(true),
                usize::MAX,
            ))
        }

        fn batch<'a>(
            &self,
            _: Batch<'a>,
        ) -> impl Future<Output = Self::MethodResponse> + Send + 'a {
            std::future::ready(MethodResponse::response(
                Id::Number(1),
                ResponsePayload::success(true),
                usize::MAX,
            ))
        }

        fn notification<'a>(
            &self,
            _: Notification<'a>,
        ) -> impl Future<Output = Self::NotificationResponse> + Send + 'a {
            std::future::ready(MethodResponse::notification())
        }
    }

    #[tokio::test]
    async fn test_rpc_auth_unauthorized_eth_success() {
        let mut auth = test_auth();
        let validator = RpcAuthMiddleware::new(MockRpcService, &vec!["brc20_hello".to_string()]);
        let mut request = hyper::Request::builder()
            .header("Authorization", "Basic asdfgh==")
            .body(HttpBody::empty())
            .unwrap();

        assert!(auth.validate(&mut request).is_ok());

        let mut rpc_request =
            jsonrpsee::types::Request::owned("eth_blockNumber".into(), None, Id::Number(1));

        rpc_request.extensions = request.extensions().clone();

        assert!(validator.call(rpc_request).await.is_success());
    }

    #[tokio::test]
    async fn test_rpc_auth_authorized_eth_success() {
        let mut auth = test_auth();
        let validator = RpcAuthMiddleware::new(MockRpcService, &vec!["brc20_hello".to_string()]);
        let mut request = hyper::Request::builder()
            .header("Authorization", test_header())
            .body(HttpBody::empty())
            .unwrap();

        assert!(auth.validate(&mut request).is_ok());
        assert!(request.extensions().get::<Authorized>().is_some());

        let mut rpc_request =
            jsonrpsee::types::Request::owned("eth_blockNumber".into(), None, Id::Number(1));

        rpc_request.extensions = request.extensions().clone();

        assert!(validator.call(rpc_request).await.is_success());
    }

    #[tokio::test]
    async fn test_rpc_auth_authorized_brc20_success() {
        let mut auth = test_auth();
        let validator = RpcAuthMiddleware::new(MockRpcService, &vec!["brc20_hello".to_string()]);
        let mut request = hyper::Request::builder()
            .header("Authorization", test_header())
            .body(HttpBody::empty())
            .unwrap();

        assert!(auth.validate(&mut request).is_ok());
        assert!(request.extensions().get::<Authorized>().is_some());

        let mut rpc_request =
            jsonrpsee::types::Request::owned("brc20_hello".into(), None, Id::Number(1));

        rpc_request.extensions = request.extensions().clone();

        assert!(validator.call(rpc_request).await.is_success());
    }

    #[tokio::test]
    async fn test_rpc_auth_wrong_credentials_brc20_error() {
        let mut auth = test_auth();
        let validator = RpcAuthMiddleware::new(MockRpcService, &vec!["brc20_hello".to_string()]);
        let mut request = hyper::Request::builder()
            .header("Authorization", "Basic asdfgh==")
            .body(HttpBody::empty())
            .unwrap();

        assert!(auth.validate(&mut request).is_ok());

        let mut rpc_request =
            jsonrpsee::types::Request::owned("brc20_hello".into(), None, Id::Number(1));

        rpc_request.extensions = request.extensions().clone();

        assert!(validator.call(rpc_request).await.is_error());
    }

    #[tokio::test]
    async fn test_rpc_auth_no_header_brc20_error() {
        let mut auth = test_auth();
        let validator = RpcAuthMiddleware::new(MockRpcService, &vec!["brc20_hello".to_string()]);
        let mut request = hyper::Request::builder().body(HttpBody::empty()).unwrap();

        assert!(auth.validate(&mut request).is_ok());

        let mut rpc_request =
            jsonrpsee::types::Request::owned("brc20_hello".into(), None, Id::Number(1));

        rpc_request.extensions = request.extensions().clone();

        assert!(validator.call(rpc_request).await.is_error());
    }

    #[tokio::test]
    async fn test_rpc_auth_no_header_eth_success() {
        let mut auth = test_auth();
        let validator = RpcAuthMiddleware::new(MockRpcService, &vec!["brc20_hello".to_string()]);
        let mut request = hyper::Request::builder().body(HttpBody::empty()).unwrap();

        assert!(auth.validate(&mut request).is_ok());

        let mut rpc_request =
            jsonrpsee::types::Request::owned("eth_yo".into(), None, Id::Number(1));

        rpc_request.extensions = request.extensions().clone();

        assert!(validator.call(rpc_request).await.is_success());
    }

    #[tokio::test]
    async fn test_deny_all_rejects_correct_credentials() {
        let mut auth = HttpNonBlockingAuth::deny();
        let validator = RpcAuthMiddleware::new(MockRpcService, &vec!["brc20_hello".to_string()]);
        let mut request = hyper::Request::builder()
            .header("Authorization", test_header())
            .body(HttpBody::empty())
            .unwrap();

        assert!(auth.validate(&mut request).is_ok());
        assert!(request.extensions().get::<Authorized>().is_none());

        let mut brc20_request =
            jsonrpsee::types::Request::owned("brc20_hello".into(), None, Id::Number(1));
        brc20_request.extensions = request.extensions().clone();
        assert!(validator.call(brc20_request).await.is_error());

        let mut eth_request =
            jsonrpsee::types::Request::owned("eth_blockNumber".into(), None, Id::Number(1));
        eth_request.extensions = request.extensions().clone();
        assert!(validator.call(eth_request).await.is_success());
    }

    #[tokio::test]
    async fn test_deny_all_without_header() {
        let mut auth = HttpNonBlockingAuth::deny();
        let mut request = hyper::Request::builder().body(HttpBody::empty()).unwrap();

        assert!(auth.validate(&mut request).is_ok());
        assert!(request.extensions().get::<Authorized>().is_none());
    }
}
