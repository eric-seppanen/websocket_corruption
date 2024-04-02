use worker::{console_log, event, Env, Request, Response, RouteContext, Router};

mod durable;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> worker::Result<Response> {
    console_error_panic_hook::set_once();

    let router = Router::new();
    router.get_async("/connect", handler).run(req, env).await
}

/// Handle incoming connection by tunnel host.
async fn handler(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    console_log!("host connected");

    let namespace = ctx.durable_object("TUNNEL")?;

    let id = namespace.id_from_name("1234")?;
    let stub = id.get_stub()?;
    stub.fetch_with_request(req).await
}
