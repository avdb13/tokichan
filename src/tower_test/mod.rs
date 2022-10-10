#[derive(Clone)]
struct RequestHandler {}

struct Timeout<T> {
    inner_handle: T,
    duration: Duration,
}

impl<T> Handler for Timeout<T>
where
    T: Handler,
{
    type Future = Pin<Box<dyn Future<Output = Result<HttpResponse, Error>>>>;
}

impl Handler for RequestHandler {
    type Future = Pin<Box<dyn Future<Output = Result<HttpResponse, Error>>>>;

    fn call(&mut self, request: HttpRequest) -> Self::Future {
        Box::pin(async move {
            if request.path() == "/" {
                Ok(HttpResponse::ok("hello"))
            } else {
                Ok(HttpResponse::not_found())
            }
        })
    }
}

impl<R, T> Handler for Timeout<T>
where
    R: 'static,
    T: Handler<R> + Clone + 'static,
    T::Error: From<tokio::time::error::Elapsed>,
{
    type Response = T::Response;
    type Error = T::Error;
    type Future = Pin<Box<dyn Future<Output = Result<T::Response, T::Error>>>>

    fn call(&mut self, request: R) -> Self::Future {
        let mut this = self.clone();

        Box::pin(async move {
            let result = tokio::time::timeout(
                this.duration,
                this.inner_handle(request),
                ).await;

            match result {
                Ok(Ok(response)) => Ok(response),
                Ok(Err(error)) => Err(error),
                Err(elapsed) => {
                    Err(T::Error::from(elapsed))
                }
            }
        })
    }
}


impl<R, T> Handler<R> for JsonContentType<T>
where
    R: 'static,
    T: Handler<R, Response = HttpResponse> + Clone + 'static,
{
    type Response = HttpResponse;
    type Error = T::Error;
    type Future = Pin<Box<dyn Future<Output = Result<T::Response, T::Error>>>>

    fn call(&mut self, request: R) -> Self::Future {
        let mut this = self.clone();

        Box::pin(async move {
            let response = this.inner_handle(request).await?;
            response.set_header("foo", "bar");
            Ok(response)
        })
    }
}


