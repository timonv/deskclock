use anyhow::Result;
use chrono::Timelike as _;
use google_calendar3::api::{EventDateTime, Events};
use google_calendar3::hyper::client::HttpConnector;
use google_calendar3::hyper_rustls::HttpsConnector;
use google_calendar3::{chrono, hyper, hyper_rustls, oauth2, CalendarHub};
use log::{error, info};
use std::default::Default;

use crate::oauth_browser_delegate::InstalledFlowBrowserDelegate;

pub struct GoogleCalendar {
    inner: AsyncCalendar,
    runtime: tokio::runtime::Runtime,
    pub account_email: String,
}

#[derive(Debug, PartialEq)]
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
    pub fn try_new(account_email: &str) -> Result<Self> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        Ok(Self {
            inner: AsyncCalendar::default(),
            runtime,
            account_email: account_email.to_string(),
        })
    }

    pub fn get_events(&mut self) -> Vec<CalendarEvent> {
        let events = match self
            .runtime
            .block_on(self.inner.get_events(&self.account_email))
        {
            Ok(events) => events.items.unwrap_or_default(),
            Err(e) => {
                error!("Failed to get events: {}", e);
                vec![]
            }
        };
        events
            .into_iter()
            .map(|event| {
                CalendarEvent {
                    // TODO: Sane defaults
                    start: extract_date_or_default(event.start).into(),
                    end: extract_date_or_default(event.end).into(),
                    summary: event.summary.unwrap_or_else(|| "No summary".to_string()),
                }
            })
            .collect()
    }
}

fn extract_date_or_default(date_time: Option<EventDateTime>) -> chrono::DateTime<chrono::Utc> {
    date_time
        .and_then(|dt| dt.date_time)
        .unwrap_or_else(chrono::Utc::now)
}

impl AsyncCalendar {
    async fn authenticate(&mut self) -> Result<()> {
        let credentials_json = include_str!("../credentials.json");
        let creds = oauth2::parse_service_account_key(credentials_json)?;
        let auth = oauth2::ServiceAccountAuthenticator::builder(creds)
            .build()
            .await?;

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

        Ok(())
    }
    async fn get_events(&mut self, account_email: &str) -> Result<Events> {
        if self.hub.is_none() {
            self.authenticate().await?;
        }

        let time_min = chrono::offset::Local::now()
            .with_hour(0)
            .expect("Could not set min time for calendar")
            .to_utc();
        log::warn!(
            "Getting events from Google Calendar API between {} and now",
            time_min
        );
        // list calendars
        let events = self
            .hub
            .as_ref()
            .expect("Not authenticated")
            .events()
            .list(account_email)
            .single_events(true)
            .show_deleted(false)
            .order_by("startTime")
            .time_min(time_min)
            // .time_max(chrono::Utc::now())
            .max_results(20)
            .doit()
            .await?
            .1;

        Ok(events)
    }
}
