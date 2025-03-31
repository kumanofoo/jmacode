use clap::Parser;
use jma::area::{Areas, JmaAreaClass};
use jma::forecast::JmaForecast;
use jma::forecast_area::ForecastArea;
use std::env;
use std::path::Path;

async fn search_temperature_point_code(args: &Args) {
    let areas = Areas::new().await.unwrap();
    let city_class20_list = areas.search_class20s(&args.city);

    if city_class20_list.is_empty() {
        let param: Vec<String> = env::args().collect();
        let program_name = Path::new(&param[0])
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();

        eprintln!(
            "{}: city '{}' not found in area codes",
            program_name, args.city
        );
        return;
    }

    let mut unique = Vec::new();
    let forecast_area_db = ForecastArea::new().await.unwrap();
    for city_class20 in city_class20_list {
        if args.verbose {
            eprintln!("found {}({})", city_class20.area.name, city_class20.code);
        }
        let city_class10 = match areas.ancestor(&city_class20, &JmaAreaClass::Class10) {
            Some(area) => area,
            None => {
                eprintln!("Error: class10 parents of '{:?}' not found", city_class20);
                continue;
            }
        };
        let city_office = match areas.ancestor(&city_class20, &JmaAreaClass::Office) {
            Some(area) => area,
            None => {
                eprintln!("Error: office parents of '{:?}' not found", city_class20);
                continue;
            }
        };
        let amedas = match forecast_area_db.get_amedas_by_class10(&city_class10.code) {
            Some(a) => a,
            None => continue,
        };
        let forecast = JmaForecast::new(&city_office.code).await.unwrap();
        let temperature_points = forecast.get_temperature_points();
        let mut results = Vec::new();
        for point in temperature_points {
            if amedas.contains(&point.area.code) {
                if args.verbose {
                    eprintln!("temperature point: {}", point.area.name);
                }
                results.push(point.area.clone());
            }
        }
        if results.is_empty() {
            eprintln!(
                "Error: {} has no temperature points",
                city_class20.area.name
            );
        }
        for res in results {
            if unique.contains(&res.code) {
                continue;
            } else {
                unique.push(res.code.clone());
            }
            println!("[area]");
            println!(
                "    jma_offices = \"{}\"     # {}",
                city_office.code, city_office.area.name
            );
            println!("    jma_area_code = \"{}\"    # {}", res.code, res.name);
            println!("    reference_time = 5");
        }
    }
}

async fn search_code(args: &Args) {
    let areas = Areas::new().await.unwrap();
    let area_list = areas.search(&args.city);

    if area_list.is_empty() {
        let param: Vec<String> = env::args().collect();
        let program_name = Path::new(&param[0])
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();

        eprintln!(
            "{}: city '{}' not found in area codes",
            program_name, args.city
        );
        return;
    }

    println!("keyword: {}", args.city);
    println!("--");
    for area in area_list {
        println!("{{");
        println!("  \"name\": \"{}\",", area.area.name);
        println!("  \"en_name\": \"{}\",", area.area.en_name);
        if let Some(p) = area.area.parent {
            println!("  \"parent\": \"{}\",", p);
        }
        if let Some(office_name) = area.area.office_name {
            println!("  \"office_name\": \"{}\",", office_name);
        }
        if let Some(children) = area.area.children {
            println!("  \"children\": {:?},", children);
        }
        println!("  \"class\": \"{}\",", area.class);
        println!("  \"code\": \"{}\",", area.code);
        println!("}}");
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help = "City Name")]
    city: String,

    #[arg(short, long, help = "Display a lot of information")]
    verbose: bool,

    #[arg(short, long, help = "Search Temperature Point Code")]
    temperature: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.verbose {
        eprintln!("searching '{}'", args.city);
    }

    if args.temperature {
        search_temperature_point_code(&args).await;
    } else {
        search_code(&args).await;
    }
}
