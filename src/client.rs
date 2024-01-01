use anyhow::{Context, Result};
use headless_chrome::{
    browser::tab::RequestPausedDecision,
    protocol::cdp::Fetch::{events::RequestPausedEvent, RequestPattern},
    Browser, LaunchOptionsBuilder,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Lots of shenanigans since we can't directly set the headers inside the Fn interceptor because it's not FnMut.
use std::sync::mpsc::{channel, Receiver, Sender};
type MutexedHeaderSender = Mutex<Sender<String>>;
type MutexedHeaderReceiver = Mutex<Receiver<String>>;
static HEADER_CHANNEL: Lazy<(MutexedHeaderSender, MutexedHeaderReceiver)> = Lazy::new(|| {
    let (tx, rx) = channel();
    (Mutex::new(tx), Mutex::new(rx))
});

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Client {
    pub cookie: String,
}

impl Client {
    const FILENAME: &'static str = "client.bincode";

    pub fn new() -> Result<Self> {
        // First, try to read a serialized version from file.
        if let Ok(file) = std::fs::File::open(Self::FILENAME) {
            let client: Self = bincode::deserialize_from(&file)?;

            // Try an HTTP request to make sure the unserialized client has correct data.
            if let Ok(response) = ureq::get("https://tos.churchofjesuschrist.org/api/appointments")
                .set("Cookie", &client.cookie)
                .call()
            {
                if response.status() == 200 {
                    return Ok(client);
                }
            }
        }

        let username =
            std::env::var("USERNAME").context("Unable to get environment variable USERNAME")?;
        let password =
            std::env::var("PASSWORD").context("Unable to get environment variable PASSWORD")?;

        // That didn't work, so go log in.
        let launch_options = LaunchOptionsBuilder::default().headless(true).build()?;
        let browser = Browser::new(launch_options)?;

        let tab = browser.new_tab()?;

        tab.navigate_to(
            "https://www.churchofjesuschrist.org/temples/schedule/appointment?lang=eng",
        )?;

        // Username. There's probably a better way to do this than clicking the element 3 times, but just doing it
        // once seems to fail on slow internet connections.
        for _ in 0..3 {
            tab.wait_for_element("input#okta-signin-username")?
                .click()?;
        }

        tab.type_str(&username)?;
        tab.wait_for_element("input#okta-signin-submit")?.click()?;

        // Password
        tab.wait_for_element("input[type=password]")?.click()?;
        tab.type_str(&password)?;

        std::thread::sleep(Duration::from_secs(1)); // Not pausing here sometimes results in crashes.

        tab.wait_for_element("input[type=submit]")?.click()?;
        std::thread::sleep(Duration::from_secs(15));

        tab.navigate_to("https://tos.churchofjesuschrist.org/?lang=eng")?;

        tab.wait_for_element("button#select-this-temple-button")?
            .click()?;

        let items = tab.wait_for_elements("span.schedule-item-text")?;

        assert!(items.len() == 4, "didn't find enough schedule items");

        // Get the info we need to start requesting stuff ourselves.
        let pattern = RequestPattern {
            url_pattern: None,
            resource_Type: Some(headless_chrome::protocol::cdp::Network::ResourceType::Xhr),
            request_stage: Some(headless_chrome::protocol::cdp::Fetch::RequestStage::Request),
        };

        let interceptor = Arc::new(|_, _, event: RequestPausedEvent| {
            let request = event.params.request;
            if request.url
                == "https://tos.churchofjesuschrist.org/api/templeSchedule/getSessionInfo"
                && request.method == "POST"
            {
                if let Some(serde_json::Value::Object(json_headers)) = request.headers.0 {
                    if let Some(serde_json::value::Value::String(cookie)) =
                        json_headers.get("Cookie")
                    {
                        HEADER_CHANNEL
                            .0
                            .lock()
                            .unwrap()
                            .send(cookie.to_string())
                            .unwrap();
                    }
                }
            }
            RequestPausedDecision::Continue(None)
        });

        tab.enable_fetch(Some(&[pattern]), None)?;
        tab.enable_request_interception(interceptor)?;

        let endowment_item = &items[2];
        endowment_item.click()?;

        let cookie = HEADER_CHANNEL.1.lock().unwrap().recv().unwrap();

        let client = Self { cookie };

        let file = std::fs::File::create(Self::FILENAME)?;
        bincode::serialize_into(file, &client)?;

        Ok(client)
    }
}
