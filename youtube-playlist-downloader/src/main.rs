use std::collections::HashMap;
use std::process::Command;
use std::fs;

use google_youtube3::{YouTube, Result, oauth2, hyper, hyper_rustls};
use std::default::Default;
use clap::{App, Arg};

fn download(name: &str, url: &str) {
    // Create a directory
    match fs::create_dir(name) {
        Err(why) => { println!("! {:?}", why.kind()) ; return },
        Ok(_) => {},
    }

    Command::new("yt-dlp")
        .current_dir(name)
        .arg(url)
        .status()
        .expect("Failed to execute yt-dlp");
}

/*
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key and playlist ID from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <api_key> <playlist_id>", args[0]);
        std::process::exit(1);
    }
    let api_key = &args[1];
    let playlist_id = &args[2];

    // Initialize the YouTube client with the API key
    let youtube = YouTube::new(api_key);

    // Call the API to retrieve playlist items
    let response = youtube.playlist_items().list("snippet")
        .playlist_id(playlist_id)
        .max_results(50) // Maximum number of results per page
        .doit();

    let mut playlist = HashMap::new();

    // Iterate over the playlist items and print titles and URLs
    for item in response.items.unwrap_or_default() {
        let snippet = item.snippet.unwrap();
        if let Some(title) = snippet.title {
            if let Some(resource) = snippet.resource {
                if let Some(video_id) = resource.id {
                    let video_url = format!("https://www.youtube.com/watch?v={}", video_id);

                    playlist.insert(title, video_url);

                    println!("Title: {}", title);
                    println!("URL: {}\n", video_url);
                }
            }
        }
    }

    for (video, &url) in playlist.iter() {
        println!("Downloading {}", video);
        download(playlist, url);
    }

    Ok(())
}
*/

//fn main() -> Result<(), Box<dyn std::error::Error>> {
#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("YouTube Playlist Viewer")
        .version("1.0")
        .author("Your Name")
        .about("Displays video titles and URLs from YouTube playlists")
        .arg(
            Arg::with_name("api-key")
                .short('k')
                .long("api-key")
                .value_name("API_KEY")
                .help("Sets the YouTube API key")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("channel-name")
                .short('c')
                .long("channel")
                .value_name("CHANNEL_NAME")
                .help("Sets the name of the YouTube channel")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("download")
                .short('d')
                .long("download")
                .value_name("DOWNLOAD")
                .help("Download videos")
                .takes_value(false)
        )
        .get_matches();

    let api_key = matches.value_of("api-key").unwrap();
    let channel_name = matches.value_of("channel-name").unwrap();

    //let youtube = YouTube::new(api_key);

    // Get an ApplicationSecret instance by some means. It contains the `client_id` and
    // `client_secret`, among other things.
    let secret: oauth2::ApplicationSecret = Default::default();

    // Instantiate the authenticator. It will choose a suitable authentication flow for you,
    // unless you replace  `None` with the desired Flow.
    // Provide your own `AuthenticatorDelegate` to adjust the way it operates and get feedback about
    // what's going on. You probably want to bring in your own `TokenStorage` to persist tokens and
    // retrieve them from storage.
    let auth = oauth2::InstalledFlowAuthenticator::builder(
        secret,
        oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    ).build().await.unwrap();

    let youtube = YouTube::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().build()), auth);

    // Get playlists for the specified channel
    let (_, results) = youtube.playlists().list(&vec!["snippet".into()])
        .channel_id(channel_name)
        .doit()
        .await
        .unwrap_or_default();

    let playlists = results.items;

    let mut videos: HashMap<String, (String, String)> = HashMap::new();

    // Iterate over playlists
    for p in playlists {
        for playlist in p {
            let playlist_title = playlist.snippet.unwrap().title.unwrap();
            let playlist_id = playlist.id.unwrap();

            println!("Playlist: {}", playlist_title);

            // Get playlist items
            let (_, playlist_items_results) = youtube.playlist_items().list(&vec!["snippet".into()])
                .playlist_id(&playlist_id)
                .max_results(50)
                .doit()
                .await
                .unwrap_or_default();

            let playlist_items = playlist_items_results.items;

            // Iterate over playlist items
            for items in playlist_items {
                for item in items {
                    if let Some(title) = item.snippet.clone().unwrap().title {
                        if let Some(resource) = item.snippet.unwrap().resource_id {
                            if let Some(video_id) = resource.video_id {
                                let video_url = format!("https://www.youtube.com/watch?v={}", video_id);
                                videos.insert(video_url.clone(), (playlist_title.clone(), title.clone()));

                                println!("Title: {}", title);
                                println!("URL: {}\n", video_url);
                            }
                        }
                    }
                }
            }
        }
    }

    // Download all playlist videos
    if matches.is_present("download") {
        for (video_url, (playlist_title, title)) in videos.iter() {
            println!("[ {} ] Downloading {}", playlist_title, title);
            download(playlist_title, video_url);
        }
    }

    Ok(())
}
