use worker::*;
use punycode::decode;

mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);
    utils::set_panic_hook();

    let router = Router::new();
    router
        // return subdomain html
        .get("/", |req, _| {
            let host = req.headers().get("host").unwrap_or_default();
            match host {
                Some(host) => {
                    let (subdomain, host) = parse_host(host.to_string());
                    let subdomain_punycode = convert_punycode(subdomain);
                    let title = format!("{}ねすぎた", subdomain_punycode);
                    let message = format!("{}おきれなかった...", subdomain_punycode);

                    let html = create_html(title, message, host);
                    Response::from_html(html)
                }
                None => Response::ok("Invalid host header."),
            }
        })

        .get("/worker-version", |_, ctx| {
            let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
            Response::ok(version)
        })

        .get("/slept.png", |_, _| {
            let favicon = include_bytes!("../static/slept.png");
            Response::from_bytes(favicon.to_vec())
        })

        .run(req, env)
        .await
}

fn parse_host(host: String) -> (String, String) {
    let mut subdomain = String::new();
    let domain = host;
    if domain.contains(".over-sle.pt") {
        subdomain = domain.replace(".over-sle.pt", "");
    }
    (subdomain, domain)
}

fn convert_punycode(sub: String) -> String {
    let mut subdomain = sub;
    if subdomain.contains("xn--") {
        subdomain = subdomain.replace("xn--", "");
        subdomain = decode(&subdomain).unwrap_or_default();
    }
    subdomain
}

fn create_html(title: String, message: String, domain: String) -> String {
    let html = include_str!("../static/index.html.tmpl");
    html.replace("{{ .Title }}", &title)
        .replace("{{ .Message }}", &message)
        .replace("{{ .Domain }}", &domain)
}