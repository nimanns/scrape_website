use reqwest::blocking::*;

#[derive(Debug)]
struct Project {
    url: Option<String>,
    image: Option<String>,
    name: Option<String>,
}

fn main() {
    let response = get("https://nimaniazi.com");    
    let html_content = response.unwrap().text().unwrap();

    let document = scraper::Html::parse_document(&html_content);
    let html_project_selector = scraper::Selector::parse("div#four>div.content>div.card").unwrap();
    let html_projects = document.select(&html_project_selector);
    let mut projects : Vec<Project> = Vec::new();

    for project in html_projects {
        let mut url = project
            .select(&scraper::Selector::parse("a").unwrap())
            .next()
            .and_then(|a| a.value().attr("href"))
            .map(str::to_owned);
    
        if let Some(ref mut url_str) = url {
            if !url_str.starts_with("https://") {
                *url_str = format!("https://nimaniazi.com{}", url_str);
            }
        }

        let mut image = project
            .select(&scraper::Selector::parse("a>img").unwrap())
            .next()
            .and_then(|img| img.value().attr("src"))
            .map(str::to_owned);

        if let Some(ref mut image_str) = image {
            *image_str = format!("https://nimaniazi.com{}", image_str);
        }

        println!("{:?}", image);

        let mut name = project
            .select(&scraper::Selector::parse("h3").unwrap())
            .next()
            .map(|h3| h3.text().collect::<String>());
        
        if let Some(ref mut name_str) = name {
            *name_str = name_str.trim().to_string();
        }

        let project = Project {url, image, name};
        projects.push(project);
    }

    for project in projects {
        println!("{:?}", project);
    }

}
