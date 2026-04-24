use crate::ContentRating;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CharacterCardV2 {
    pub spec: String,
    pub spec_version: String,
    pub data: CharacterCardV2Data,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CharacterCardV2Data {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub personality: String,
    #[serde(default)]
    pub scenario: String,
    #[serde(default)]
    pub first_mes: String,
    #[serde(default)]
    pub mes_example: String,
    #[serde(default)]
    pub creator_notes: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub extensions: serde_json::Value,
}

impl CharacterCardV2Data {
    /// Extracts content rating from extensions, defaults to Pg.
    pub fn content_rating(&self) -> ContentRating {
        self.extensions
            .get("content_rating")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(ContentRating::Pg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ContentRating;

    #[test]
    fn parse_minimal_character_card_v2() {
        let json = r#"{
        "spec": "chara_card_v2",
        "spec_version": "2.0",
        "data": {
            "name": "Aria",
            "description": "A helpful assistant",
            "personality": "Calm and thoughtful",
            "scenario": "A quiet library",
            "first_mes": "Hello, how can I help?",
            "mes_example": "",
            "creator_notes": "",
            "tags": []
        }
    }"#;
        let card: CharacterCardV2 = serde_json::from_str(json).unwrap();
        assert_eq!(card.data.name, "Aria");
        assert_eq!(card.data.content_rating(), ContentRating::Pg);
    }

    #[test]
    fn parse_card_with_nsfw_extension() {
        let json = r#"{
          "spec": "chara_card_v2",
          "spec_version": "2.0",
          "data": {
              "name": "Aria",
              "description": "",
              "personality": "",
              "scenario": "",
              "first_mes": "",
              "mes_example": "",
              "creator_notes": "",
              "tags": [],
              "extensions": { "content_rating": "nsfw" }
          }
      }"#;

        let card: CharacterCardV2 = serde_json::from_str(json).unwrap();
        assert_eq!(card.data.content_rating(), ContentRating::Nsfw);
    }

    #[test]
    fn reject_non_v2_card() {
        let json = r#"{
      "spec": "chara_card_v1",
      "spec_version": "1.0",
      "data": {
        "name": "Old Card"
      }
    }"#;
        // On attend une erreur de validation, pas de parsing
        // (la validation se fera dans la couche import, pas ici)
        // Ce test vérifie uniquement que le parsing ne panique pas
        let result: Result<CharacterCardV2, _> = serde_json::from_str(json);
        assert!(result.is_ok()); // serde parse, la validation est ailleurs
    }
}
