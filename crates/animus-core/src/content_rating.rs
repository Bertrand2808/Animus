use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentRating {
    Pg,
    Mature,
    Nsfw
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
            other => Err(ContentRatingParseError(other.to_owned()))
        }
    }
}

// TODO: Implémenter Display pour ContentRating
// - Permet d'afficher facilement le rating de façon lisible pour l'utilisateur
//   (ex: dans la liste des personnages sur React, les warnings NSFW, les tooltips, les filtres...)
// - Utile dans les logs du backend Rust ("Character 'Waifu X' loaded with rating: {}")
// - On pourra faire `format!("{}", rating)` ou `rating.to_string()` partout sans boilerplate
// - Plus propre et plus "Rust idiomatique" que de dépendre uniquement de serde (qui donne du lowercase)
// - Ça rendra le frontend React plus simple : on pourra renvoyer une string propre depuis l'API
//   au lieu de mapper manuellement les valeurs.

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
