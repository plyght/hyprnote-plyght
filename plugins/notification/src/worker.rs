use apalis::prelude::{Data, Error, WorkerBuilder, WorkerFactoryFn};
use chrono::{DateTime, Duration, Utc};
use hypr_db_user::{Event, ListEventFilter, ListEventFilterCommon, ListEventFilterSpecific};

#[allow(unused)]
#[derive(Default, Debug, Clone)]
pub struct Job(DateTime<Utc>);

#[derive(Clone)]
pub struct WorkerState {
    pub db: hypr_db_user::UserDatabase,
    pub user_id: String,
}

impl From<DateTime<Utc>> for Job {
    fn from(t: DateTime<Utc>) -> Self {
        Job(t)
    }
}

const EVENT_NOTIFICATION_WORKER_NAKE: &str = "event_notification_worker";

#[derive(Debug, Clone)]
enum MeetingType {
    InPerson,
    Virtual(String), // Contains the app name/platform
}

fn detect_meeting_type(event: &Event) -> MeetingType {
    if let Some(url) = &event.google_event_url {
        if url.contains("zoom.us") || url.contains("/j/") {
            return MeetingType::Virtual("zoom".to_string());
        }
        if url.contains("teams.microsoft.com") {
            return MeetingType::Virtual("teams".to_string());
        }
        if url.contains("meet.google.com") {
            return MeetingType::Virtual("meet".to_string());
        }
        if url.contains("webex.com") {
            return MeetingType::Virtual("webex".to_string());
        }
        if url.contains("gotomeeting.com") {
            return MeetingType::Virtual("gotomeeting".to_string());
        }
        if url.contains("bluejeans.com") {
            return MeetingType::Virtual("bluejeans".to_string());
        }
        if url.contains("join.me") {
            return MeetingType::Virtual("gotomeeting".to_string());
        }
        if url.contains("meet.") || url.contains("call.") {
            return MeetingType::Virtual("meet".to_string());
        }
    }

    // Check event name for virtual meeting indicators
    let name_lower = event.name.to_lowercase();
    if name_lower.contains("zoom") {
        return MeetingType::Virtual("zoom".to_string());
    }
    if name_lower.contains("teams") {
        return MeetingType::Virtual("teams".to_string());
    }
    if name_lower.contains("meet") || name_lower.contains("google meet") {
        return MeetingType::Virtual("meet".to_string());
    }
    if name_lower.contains("webex") {
        return MeetingType::Virtual("webex".to_string());
    }
    if name_lower.contains("call")
        || name_lower.contains("virtual")
        || name_lower.contains("online")
    {
        return MeetingType::Virtual("meet".to_string()); // Default to generic meet
    }

    MeetingType::InPerson
}

#[tracing::instrument(skip(ctx), name = EVENT_NOTIFICATION_WORKER_NAKE)]
pub async fn perform_event_notification(_job: Job, ctx: Data<WorkerState>) -> Result<(), Error> {
    let latest_event = ctx
        .db
        .list_events(Some(ListEventFilter {
            common: ListEventFilterCommon {
                user_id: ctx.user_id.clone(),
                limit: Some(1),
            },
            specific: ListEventFilterSpecific::DateRange {
                start: Utc::now(),
                end: Utc::now() + Duration::minutes(5),
            },
        }))
        .await
        .map_err(|e| crate::Error::Db(e).as_worker_error())?;

    if let Some(event) = latest_event.first() {
        let meeting_type = detect_meeting_type(event);
        let (title, message, icon) = match meeting_type {
            MeetingType::Virtual(platform) => (
                "Event with video link".to_string(),
                format!("{}\nClick to start listening & take notes.", event.name),
                Some(hypr_notification2::NotificationIcon::AppIcon(platform)),
            ),
            MeetingType::InPerson => (
                "Event".to_string(),
                format!("{}\nClick to start taking notes.", event.name),
                Some(hypr_notification2::NotificationIcon::Calendar),
            ),
        };

        hypr_notification2::show(hypr_notification2::Notification {
            title,
            message,
            url: Some(format!("hypr://notification?event_id={}", event.id)),
            timeout: Some(std::time::Duration::from_secs(10)),
            icon,
        });
    }

    Ok(())
}

pub async fn monitor(state: WorkerState) -> Result<(), std::io::Error> {
    use std::str::FromStr;

    apalis::prelude::Monitor::new()
        .register({
            WorkerBuilder::new(EVENT_NOTIFICATION_WORKER_NAKE)
                .data(state.clone())
                .backend(apalis_cron::CronStream::new(
                    apalis_cron::Schedule::from_str("0 * * * * *").unwrap(),
                ))
                .build_fn(perform_event_notification)
        })
        .run()
        .await?;

    Ok(())
}
