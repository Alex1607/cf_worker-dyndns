use std::collections::HashMap;

use serde_json::json;
use worker::*;
use worker::wasm_bindgen::{JsValue, UnwrapThrowExt};

const NAME: &str = "n";

#[event(fetch)]
async fn main(req: Request, env: Env, _: Context) -> Result<Response> {
    let router = Router::new();

    router
        .get_async("/", |req, ctx| async move {
            let secret_apikey = ctx.secret("apikey");
            let secret_cloudflare_apikey = ctx.secret("cloudflare_apikey");
            let zone_identifier = ctx.var("zone_identifier");
            let identifier = ctx.var("identifier");

            if secret_apikey.is_err() || zone_identifier.is_err() || identifier.is_err() || secret_cloudflare_apikey.is_err() {
                return Response::error("Unable to get secrets / and variables", 500);
            }

            let hash_query: HashMap<_, _> = req.url()?.query_pairs().into_owned().collect();
            let ip4 = hash_query.get("ipv4");
            let ip6 = hash_query.get("ipv6");

            let Some(api_key) = hash_query.get("apikey") else {
                return Response::error("Request is missing the APIKey", 400);
            };

            if secret_apikey.unwrap().to_string().ne(api_key) {
                return Response::error("APIKey invalid", 403);
            }

            let url = Url::parse(
                format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                        zone_identifier.unwrap().to_string(),
                        identifier.unwrap().to_string())
                    .as_str()
            );

            let content = ip6.unwrap_or_else(|| ip4.unwrap_throw());
            let dns_type = if ip6.is_some() { "AAAA" } else { "A" };

            let data = json!({
                "content": content,
                "name": NAME,
                "type": dns_type
            }).to_string();

            let mut headers = Headers::new();
            headers.set("Content-Type", "application/json")?;
            headers.set("Authorization", secret_cloudflare_apikey.unwrap().to_string().as_str())?;

            let mut request_init = RequestInit::new();
            request_init
                .with_method(Method::Put)
                .with_body(Some(JsValue::from(data)))
                .with_headers(headers)
                .with_redirect(RequestRedirect::Follow);

            let request = Request::new_with_init(url?.as_str(), &request_init)?;
            let response = Fetch::Request(request).send().await?;

            if !(200..299).contains(&response.status_code()) {
                return Response::error("Unable to update DNS", 500);
            }

            Response::ok("")
        })
        .run(req, env)
        .await
}
