use actix_web::{web, App, HttpServer, HttpResponse};
use reqwest;
use scraper::{Html, Selector};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct Project {
    url: Option<String>,
    image: Option<String>,
    name: Option<String>,
}

async fn fetch_projects() -> Vec<Project> {
    let response = reqwest::get("https://nimaniazi.com")
        .await
        .unwrap();
    let html_content = response.text().await.unwrap();

    let document = Html::parse_document(&html_content);
    let html_project_selector = Selector::parse("div#four>div.content>div.card").unwrap();
    let html_projects = document.select(&html_project_selector);
    let mut projects: Vec<Project> = Vec::new();

    for project in html_projects {
        let mut url = project
            .select(&Selector::parse("a").unwrap())
            .next()
            .and_then(|a| a.value().attr("href"))
            .map(str::to_owned);
    
        if let Some(ref mut url_str) = url {
            if !url_str.starts_with("https://") {
                *url_str = format!("https://nimaniazi.com{}", url_str);
            }
        }

        let mut image = project
            .select(&Selector::parse("a>img").unwrap())
            .next()
            .and_then(|img| img.value().attr("src"))
            .map(str::to_owned);

        if let Some(ref mut image_str) = image {
            *image_str = format!("https://nimaniazi.com{}", image_str);
        }

        let mut name = project
            .select(&Selector::parse("h3").unwrap())
            .next()
            .map(|h3| h3.text().collect::<String>());
        
        if let Some(ref mut name_str) = name {
            *name_str = name_str.trim().to_string();
        }

        let project = Project { url, image, name };
        projects.push(project);
    }

    projects
}

async fn index() -> HttpResponse {
    let projects = fetch_projects().await;
    HttpResponse::Ok()
        .content_type("application/json")
        .json(projects)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

