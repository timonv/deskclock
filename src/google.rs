// extern crate google_calendar3 as calendar3;
// extern crate hyper;
// extern crate hyper_rustls;
use anyhow::Result;
use chrono::Duration;
use google_calendar3::api::{Channel, Event, EventDateTime, Events};
use google_calendar3::hyper::client::HttpConnector;
use google_calendar3::hyper_rustls::HttpsConnector;
use google_calendar3::Error;
use google_calendar3::{chrono, hyper, hyper_rustls, oauth2, CalendarHub, FieldMask};
use log::info;
use std::default::Default;

use crate::oauth_browser_delegate::InstalledFlowBrowserDelegate;

pub struct GoogleCalendar {
    inner: AsyncCalendar,
    runtime: tokio::runtime::Runtime,
}

pub struct CalendarEvent {
    pub start: chrono::DateTime<chrono::Local>,
    pub end: chrono::DateTime<chrono::Local>,
    pub summary: String,
}

#[derive(Default)]
struct AsyncCalendar {
    hub: Option<CalendarHub<HttpsConnector<HttpConnector>>>,
}

impl GoogleCalendar {
    pub fn try_new() -> Result<Self> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        Ok(Self {
            inner: AsyncCalendar::default(),
            runtime,
        })
    }

    pub fn authenticate(&mut self) {
        self.runtime.block_on(self.inner.authenticate())
    }

    pub fn get_events(&self) -> Vec<CalendarEvent> {
        let events = self.runtime.block_on(self.inner.get_events()).unwrap();
        events.items.map_or(vec![], |items| {
            items
                .into_iter()
                .map(|event| CalendarEvent {
                    // TODO: Sane defaults
                    start: extract_date_or_default(event.start).into(),
                    end: extract_date_or_default(event.end).into(),
                    summary: event.summary.unwrap_or_else(|| "No summary".to_string()),
                })
                .collect()
        })
    }
}

fn extract_date_or_default(date_time: Option<EventDateTime>) -> chrono::DateTime<chrono::Utc> {
    date_time
        .and_then(|dt| dt.date_time)
        .unwrap_or_else(chrono::Utc::now)
}

impl AsyncCalendar {
    async fn authenticate(&mut self) {
        info!("Authenticating with Google Calendar API");
        // Get an ApplicationSecret instance by some means. It contains the `client_id` and
        // `client_secret`, among other things.
        // let secret: oauth2::ApplicationSecret = Default::default();
        // Read secret from file
        let secret = oauth2::read_application_secret("credentials.json")
            .await
            .unwrap();
        // Instantiate the authenticator. It will choose a suitable authentication flow for you,
        // unless you replace  `None` with the desired Flow.
        // Provide your own `AuthenticatorDelegate` to adjust the way it operates and get feedback about
        // what's going on. You probably want to bring in your own `TokenStorage` to persist tokens and
        // retrieve them from storage.
        let auth = oauth2::InstalledFlowAuthenticator::builder(
            secret,
            oauth2::InstalledFlowReturnMethod::HTTPRedirect,
        )
        .persist_tokens_to_disk("tokencache.json")
        .flow_delegate(Box::new(InstalledFlowBrowserDelegate))
        .build()
        .await
        .unwrap();
        info!("Authenticated with Google Calendar API");
        self.hub = Some(CalendarHub::new(
            hyper::Client::builder().build(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_or_http()
                    .enable_http1()
                    .build(),
            ),
            auth,
        ));
    }

    async fn get_events(&self) -> Result<Events> {
        // As the method needs a request, you would usually fill it with the desired information
        // into the respective structure. Some of the parts shown here might not be applicable !
        // Values shown here are possibly random and not representative !

        // You can configure optional parameters by calling the respective setters at will, and
        // execute the final call using `doit()`.
        // Values shown here are possibly random and not representative !
        let time_min = chrono::Utc::now() - Duration::minutes(10);
        log::warn!(
            "Getting events from Google Calendar API between {} and now",
            time_min
        );
        let events = self
            .hub
            .as_ref()
            .expect("Not authenticated")
            .events()
            .list("primary")
            .single_events(true)
            .show_deleted(false)
            .order_by("startTime")
            .time_min(time_min)
            // .time_max(chrono::Utc::now())
            .max_results(10)
            .doit()
            .await?
            .1;

        Ok(events)
    }
}
