use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(feature = "ssr")] {
        #[actix_web::main]
        async fn main() -> std::io::Result<()> {
            use rql::app::*;
            use actix_files::Files;
            use actix_web::*;
            use leptos::*;
            use leptos_actix::{generate_route_list, LeptosRoutes};

            let client = if let Ok(client) = std::env::var("CLIENT") { client == "true"}
            else { false};

            let conf = get_configuration(None).await.unwrap();
            let addr = conf.leptos_options.site_addr;
            // Generate the list of routes in your Leptos App
            let routes = if client { Some(generate_route_list(|cx| view! { cx, <App/> })) }
            else { None };

            HttpServer::new(move || {
                let leptos_options = &conf.leptos_options;
                let site_root = &leptos_options.site_root;

                let mut app = App::new()
                    .route("/api/{tail:.*}", leptos_actix::handle_server_fns());
                if client {
                    app = app.leptos_routes(
                        leptos_options.to_owned(),
                        routes.to_owned().unwrap(),
                        |cx| view! { cx, <App/> },
                    ).service(Files::new("/", site_root));
                }
                    app.wrap(middleware::Compress::default())
            })
            .bind(&addr)?
            .run()
            .await
        }
    }
    else {
        #[cfg(not(feature = "ssr"))]
        pub fn main() {
            // no client-side main function
            // unless we want this to work with e.g., Trunk for pure client-side testing
            // see lib.rs for hydration function instead
        }
    }
}
