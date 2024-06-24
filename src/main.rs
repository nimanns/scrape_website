use actix_web::{web, App, HttpServer, HttpResponse, Responder, error::ErrorBadRequest};
use reqwest;
use scraper::{Html, Selector};
use serde::{Serialize, Deserialize};
use std::env;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use dotenv::dotenv;
use actix_cors::Cors;

#[derive(Debug, Serialize)]
struct Project {
    url: Option<String>,
    image: Option<String>,
    name: Option<String>,
}

#[derive(Deserialize)]
struct EmailRequest {
    sender_name: Option<String>,
    sender_email: Option<String>,
    recipient_email: Option<String>,
    subject: Option<String>,  // This remains optional
    body: Option<String>,
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

async fn send_email(email_req: web::Json<EmailRequest>) -> Result<HttpResponse, actix_web::Error> {
    // Check if required fields are provided
    let sender_name = email_req.sender_name.as_ref()
        .ok_or_else(|| ErrorBadRequest("sender_name is required"))?;
    let sender_email = email_req.sender_email.as_ref()
        .ok_or_else(|| ErrorBadRequest("sender_email is required"))?;
    let recipient_email = "contact@nimaniazi.com";
    let body = email_req.body.as_ref()
        .ok_or_else(|| ErrorBadRequest("body is required"))?;

    // Subject is optional, use a default if not provided
    let subject = email_req.subject.as_deref().unwrap_or("No Subject");

    // Append sender's email to the top of the body
    let full_body = format!("From: {} <{}>\n\n{}", sender_name, sender_email, body);

    let email = Message::builder()
        .from(format!("{} <{}>", sender_name, env::var("SMTP_USERNAME").expect("SMTP_USERNAME not set")).parse().unwrap())
        .to(recipient_email.parse().unwrap())
        .subject(subject)
        .body(full_body)
        .map_err(|e| ErrorBadRequest(format!("Failed to build email: {}", e)))?;

    let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME not set");
    let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD not set");

    let creds = Credentials::new(smtp_username, smtp_password);

    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => Ok(HttpResponse::Ok().body("Email sent successfully!")),
        Err(e) => Err(ErrorBadRequest(format!("Could not send email: {:?}", e))),
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    //dotenv().ok();

    let HOST = env::var("HOST").expect("Host not set");
    let PORT = env::var("PORT").expect("Port not set");

    env::var("SMTP_USERNAME").expect("SMTP_USERNAME not set");
    env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD not set");
    
    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("https://nimaniazi.com")
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .route("/", web::get().to(index))
            .route("/send-email", web::post().to(send_email))
    })
    .bind(format!("{}:{}", HOST, PORT))?
    .run()
    .await
}

