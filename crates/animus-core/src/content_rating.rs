use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentRating {
    Pg,
    Mature,
    Nsfw,
}

#[derive(Debug, Error)]
#[error("unknown content rating: {0}")]
pub struct ContentRatingParseError(String);

impl FromStr for ContentRating {
    type Err = ContentRatingParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pg" => Ok(Self::Pg),
            "mature" => Ok(Self::Mature),
            "nsfw" => Ok(Self::Nsfw),
            other => Err(ContentRatingParseError(other.to_owned())),
        }
    }
}

impl<'de> serde::Deserialize<'de> for ContentRating {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for ContentRating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pg => write!(f, "pg"),
            Self::Mature => write!(f, "mature"),
            Self::Nsfw => write!(f, "nsfw"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ContentRating;

    #[test]
    fn content_rating_from_str_pg() {
        let r: ContentRating = "pg".parse().unwrap();
        assert_eq!(r, ContentRating::Pg);
    }

    #[test]
    fn content_rating_from_str_nsfw() {
        let r: ContentRating = "nsfw".parse().unwrap();
        assert_eq!(r, ContentRating::Nsfw);
    }

    #[test]
    fn content_rating_from_str_unknown_defaults_pg() {
        let r: ContentRating = "unknown".parse().unwrap_or(ContentRating::Pg);
        assert_eq!(r, ContentRating::Pg);
    }

    #[test]
    fn content_rating_roundtrip_serde() {
        let json = serde_json::to_string(&ContentRating::Mature).unwrap();
        let back: ContentRating = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ContentRating::Mature);
    }
}
