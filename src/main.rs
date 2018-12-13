use web_view::*;
use clap::{Arg, App};
use handlebars::Handlebars;
use serde_json::json;

fn main() {
    let matches = App::new("nativefier")
        .version("0.0.1")
        .author("Jack Mordaunt <jackmordaunt@gmail.com>")
        .about("create native apps for your favourite site")
        .arg(Arg::with_name("title")
            .required(true)
            .takes_value(true)
            .help("title of site"))
        .arg(Arg::with_name("url")
            .required(true)
            .takes_value(true)
            .help("url of site to nativefy"))
        .get_matches();
    let title = matches.value_of("title").expect("parsing title flag");
    let url = matches.value_of("url").expect("parsing url flag");
    let content = Handlebars::new().render_template(HTML, &json!({"title": title, "href": url})).unwrap();
    web_view::builder()
        .title(title)
        .content(Content::Html(content))
        .size(800, 600)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|_webview, _arg| Ok(()))
        .run()
        .expect("running webview");
}


const HTML: &str = r#"
<!doctype html>
<html>
    <head>
        <script src="https://cdn.polyfill.io/v2/polyfill.min.js?features=Promise,fetch,Symbol,Array.prototype.@@iterator"></script> 
        <script>
            var script = document.createElement("script")
            script.setAttribute("type", "text/javascript")
            script.setAttribute("src", "https://cdn.polyfill.io/v2/polyfill.min.js?features=Promise,fetch,Symbol,Array.prototype.@@iterator")
            window.location = "{{href}}"
        </script>
    </head>
    <body>
        <h1>{{title}}</h1>
        <iframe src="{{href}}">{{title}}</iframe>
    </body>
</html>
"#;