use anyhow::Result;
use headless_chrome::{
    browser::tab::RequestInterceptionDecision,
    protocol::network::{events::RequestInterceptedEventParams, methods::RequestPattern},
    Browser, LaunchOptionsBuilder,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

type Headers = HashMap<String, String>;

// Lots of shenanigans since we can't directly set the headers inside the Fn interceptor because it's not FnMut.
use std::sync::mpsc::{channel, Receiver, Sender};
type MutexedHeaderSender = Mutex<Sender<Headers>>;
type MutexedHeaderReceiver = Mutex<Receiver<Headers>>;
static HEADER_CHANNEL: Lazy<(MutexedHeaderSender, MutexedHeaderReceiver)> = Lazy::new(|| {
    let (tx, rx) = channel();
    (Mutex::new(tx), Mutex::new(rx))
});

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Client {
    pub headers: Headers,
}

impl Client {
    const FILENAME: &'static str = "client.bincode";

    pub fn new() -> Result<Self> {
        // First, try to read a serialized version from file.
        if let Ok(file) = std::fs::File::open(Self::FILENAME) {
            let client: Self = bincode::deserialize_from(&file)?;

            // Try an HTTP request to make sure the unserialized client has correct data.
            if let Ok(response) = ureq::get("https://tos.churchofjesuschrist.org/api/appointments")
                .set("Cookie", client.headers.get("Cookie").unwrap())
                .set("X-XSRF-TOKEN", client.headers.get("X-XSRF-TOKEN").unwrap())
                .call()
            {
                if response.status() == 200 {
                    return Ok(client);
                }
            }
        }

        let username = std::env::var("USERNAME")?;
        let password = std::env::var("PASSWORD")?;

        // That didn't work, so go log in.
        let launch_options = LaunchOptionsBuilder::default()
            .headless(true)
            .build()
            .map_err(|s| anyhow::anyhow!(s))?;
        let browser = Browser::new(launch_options).map_err(|e| e.compat())?;

        let tab = browser.wait_for_initial_tab().map_err(|e| e.compat())?;

        tab.navigate_to(
            "https://www.churchofjesuschrist.org/temples/schedule/appointment?lang=eng",
        )
        .map_err(|e| e.compat())?;

        // Username. There's probably a better way to do this than clicking the element 3 times, but just doing it
        // once seems to fail on slow internet connections.
        for _ in 0..3 {
            tab.wait_for_element("input#okta-signin-username")
                .map_err(|e| e.compat())?
                .click()
                .map_err(|e| e.compat())?;
        }

        tab.type_str(&username).map_err(|e| e.compat())?;
        tab.wait_for_element("input#okta-signin-submit")
            .map_err(|e| e.compat())?
            .click()
            .map_err(|e| e.compat())?;

        // Password
        tab.wait_for_element("input[type=password]")
            .map_err(|e| e.compat())?
            .click()
            .map_err(|e| e.compat())?;
        tab.type_str(&password).map_err(|e| e.compat())?;

        std::thread::sleep(Duration::from_secs(1)); // Not pausing here sometimes results in crashes.

        tab.wait_for_element("input[type=submit]")
            .map_err(|e| e.compat())?
            .click()
            .map_err(|e| e.compat())?;

        std::thread::sleep(Duration::from_secs(15));

        tab.navigate_to("https://tos.churchofjesuschrist.org/?lang=eng")
            .map_err(|e| e.compat())?;

        tab.wait_for_element("button#select-this-temple-button")
            .map_err(|e| e.compat())?
            .click()
            .map_err(|e| e.compat())?;

        let items = tab
            .wait_for_elements("span.schedule-item-text")
            .map_err(|e| e.compat())?;

        assert!(items.len() == 4, "didn't find enough schedule items");

        // Get the info we need to start requesting stuff ourselves.
        let pattern = RequestPattern {
            url_pattern: None,
            resource_type: Some("XHR"),
            interception_stage: Some("Request"),
        };

        let interceptor = Box::new(|_, _, params: RequestInterceptedEventParams| {
            let request = params.request;
            if request.url
                == "https://tos.churchofjesuschrist.org/api/templeSchedule/getSessionInfo"
                && request.method == "POST"
            {
                HEADER_CHANNEL
                    .0
                    .lock()
                    .unwrap()
                    .send(request.headers)
                    .unwrap();
            }
            RequestInterceptionDecision::Continue
        });

        tab.enable_request_interception(&[pattern], interceptor)
            .map_err(|e| e.compat())?;

        let endowment_item = &items[2];
        endowment_item.click().map_err(|e| e.compat())?;

        let headers = HEADER_CHANNEL.1.lock().unwrap().recv().unwrap();
        if headers.is_empty() {
            return Err(anyhow::anyhow!("Header for making queries has no entries"));
        }

        let client = Self { headers };

        let file = std::fs::File::create(Self::FILENAME)?;
        bincode::serialize_into(file, &client)?;

        Ok(client)
    }
}
