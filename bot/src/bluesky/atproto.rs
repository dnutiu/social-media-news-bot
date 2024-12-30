use chrono::{DateTime, Local, Utc};
use post::NewsPost;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct BlobLinkRef {
    #[serde(rename = "$link")]
    pub _link: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Blob {
    #[serde(rename = "$type")]
    pub _type: String,
    #[serde(rename = "ref")]
    pub r#ref: BlobLinkRef,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub size: i64,
}

impl Blob {
    pub fn new(link: &str, mime_type: &str, size: i64) -> Self {
        Blob {
            _type: "blob".to_string(),
            r#ref: BlobLinkRef {
                _link: link.to_string(),
            },
            mime_type: mime_type.to_string(),
            size,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct BlobResponse {
    pub blob: Blob,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ATProtoServerCreateSession {
    pub(crate) identifier: String,
    pub(crate) password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExternalRecordEmbed {
    pub uri: String,
    pub title: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb: Option<Blob>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ATprotoRepoCreateRecordEmbed {
    #[serde(rename(serialize = "$type", deserialize = "$type"))]
    pub embed_type: String,
    pub external: ExternalRecordEmbed,
}

impl ATprotoRepoCreateRecordEmbed {
    fn new(uri: &str, title: &str, description: &str, thumb: Option<Blob>) -> Self {
        ATprotoRepoCreateRecordEmbed {
            embed_type: "app.bsky.embed.external".to_string(),
            external: ExternalRecordEmbed {
                uri: uri.to_string(),
                title: title.to_string(),
                description: description.to_string(),
                thumb,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ATprotoRepoCreateRecordRecord {
    pub text: String,
    #[serde(rename(serialize = "createdAt", deserialize = "createdAt"))]
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed: Option<ATprotoRepoCreateRecordEmbed>,
}

impl ATprotoRepoCreateRecordRecord {
    fn new(text: &str, date: DateTime<Utc>, embed: Option<ATprotoRepoCreateRecordEmbed>) -> Self {
        ATprotoRepoCreateRecordRecord {
            text: text.to_string(),
            created_at: date.to_rfc3339(),
            embed,
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ATProtoRepoCreateRecord {
    pub repo: String,
    pub collection: String,
    pub record: ATprotoRepoCreateRecordRecord,
}

impl ATProtoRepoCreateRecord {
    fn new(handle: &str, record: ATprotoRepoCreateRecordRecord) -> Self {
        ATProtoRepoCreateRecord {
            repo: handle.to_string(),
            collection: "app.bsky.feed.post".to_string(),
            record,
        }
    }
}

impl From<NewsPost> for ATProtoRepoCreateRecord {
    fn from(post: NewsPost) -> Self {
        let dt = Local::now();
        let dt_utc = DateTime::<Utc>::from_naive_utc_and_offset(dt.naive_utc(), Utc);

        ATProtoRepoCreateRecord::new(
            "",
            ATprotoRepoCreateRecordRecord::new(
                post.title.clone().unwrap_or(String::from("")).as_str(),
                dt_utc,
                Some(ATprotoRepoCreateRecordEmbed::new(
                    post.link.unwrap().as_str(),
                    post.title.unwrap().as_str(),
                    post.summary.unwrap().as_str(),
                    None,
                )),
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    #[test]
    fn test_atproto_server_create_session_serialization() -> Result<(), anyhow::Error> {
        let session = ATProtoServerCreateSession {
            identifier: "user".to_string(),
            password: "pass".to_string(),
        };

        let json = serde_json::to_string(&session)?;

        assert_eq!(json, r#"{"identifier":"user","password":"pass"}"#);

        Ok(())
    }

    #[test]
    fn test_atproto_repo_create_record_record_no_embed_serialization() -> Result<(), anyhow::Error>
    {
        let time_str = "2024-12-30T13:45:00";
        let naive_datetime = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%dT%H:%M:%S").unwrap();

        let session = ATProtoRepoCreateRecord::new(
            "nuculabs.dev",
            ATprotoRepoCreateRecordRecord::new(
                "some post",
                DateTime::from_naive_utc_and_offset(naive_datetime, Utc),
                None,
            ),
        );

        let json = serde_json::to_string(&session)?;

        assert_eq!(
            json,
            r#"{"repo":"nuculabs.dev","collection":"app.bsky.feed.post","record":{"text":"some post","createdAt":"2024-12-30T13:45:00+00:00"}}"#
        );

        Ok(())
    }

    #[test]
    fn test_atproto_repo_create_record_record_embed_serialization() -> Result<(), anyhow::Error> {
        let time_str = "2024-12-30T13:45:00";
        let naive_datetime = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%dT%H:%M:%S").unwrap();

        let session = ATProtoRepoCreateRecord::new(
            "nuculabs.dev",
            ATprotoRepoCreateRecordRecord::new(
                "some post",
                DateTime::from_naive_utc_and_offset(naive_datetime, Utc),
                Some(ATprotoRepoCreateRecordEmbed::new(
                    "https://some-news.ro/some",
                    "Some very important news",
                    "The description of the news",
                    None,
                )),
            ),
        );

        let json = serde_json::to_string(&session)?;

        assert_eq!(
            json,
            r#"{"repo":"nuculabs.dev","collection":"app.bsky.feed.post","record":{"text":"some post","createdAt":"2024-12-30T13:45:00+00:00","embed":{"$type":"app.bsky.embed.external","external":{"uri":"https://some-news.ro/some","title":"Some very important news","description":"The description of the news"}}}}"#
        );

        Ok(())
    }

    #[test]
    fn test_atproto_repo_create_record_record_embed_and_thumb_serialization(
    ) -> Result<(), anyhow::Error> {
        let time_str = "2024-12-30T13:45:00";
        let naive_datetime = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%dT%H:%M:%S").unwrap();

        let session = ATProtoRepoCreateRecord::new(
            "nuculabs.dev",
            ATprotoRepoCreateRecordRecord::new(
                "some post",
                DateTime::from_naive_utc_and_offset(naive_datetime, Utc),
                Some(ATprotoRepoCreateRecordEmbed::new(
                    "https://some-news.ro/some",
                    "Some very important news",
                    "The description of the news",
                    Some(Blob::new(
                        "bafkreiass5vjx467rdtm77ey4kkuz667wldaffq7z3nmvqxm2bwk3hiemm",
                        "image/jpeg",
                        122,
                    )),
                )),
            ),
        );

        let json = serde_json::to_string(&session)?;

        assert_eq!(
            json,
            r#"{"repo":"nuculabs.dev","collection":"app.bsky.feed.post","record":{"text":"some post","createdAt":"2024-12-30T13:45:00+00:00","embed":{"$type":"app.bsky.embed.external","external":{"uri":"https://some-news.ro/some","title":"Some very important news","description":"The description of the news","thumb":{"$type":"blob","ref":{"$link":"bafkreiass5vjx467rdtm77ey4kkuz667wldaffq7z3nmvqxm2bwk3hiemm"},"mimeType":"image/jpeg","size":122}}}}}"#
        );

        Ok(())
    }
}
