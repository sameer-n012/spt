use serde_json::Value;

pub fn print_track_pretty(json: &Value, indent_level: usize) -> String {
    return format!(
        "{}\"{}\" - {} by {}",
        "\t".repeat(indent_level),
        json["name"].as_str().unwrap_or("null"),
        json["album"]["name"].as_str().unwrap_or("null"),
        json["artists"][0]["name"].as_str().unwrap_or("null"),
    );
}

pub fn print_episode_pretty(json: &Value, indent_level: usize) -> String {
    return format!(
        "{}\"{}\" - {} by {}",
        "\t".repeat(indent_level),
        json["name"].as_str().unwrap_or("null"),
        json["show"]["name"].as_str().unwrap_or("null"),
        json["show"]["publisher"].as_str().unwrap_or("null"),
    );
}

pub fn print_track_episode_pretty(json: &Value, indent_level: usize) -> String {
    if json["type"] == "track" {
        return print_track_pretty(json, indent_level);
    } else if json["type"] == "episode" {
        return print_episode_pretty(json, indent_level);
    }

    return format!("{}Unknown Item", "\t".repeat(indent_level),);
}

pub fn print_track(json: &Value, indent_level: usize) -> String {
    print_track_episode(json, indent_level)
}

pub fn print_episode(json: &Value, indent_level: usize) -> String {
    print_track_episode(json, indent_level)
}

pub fn print_track_episode(json: &Value, indent_level: usize) -> String {
    return format!(
        "{}{}",
        "\t".repeat(indent_level),
        json["uri"].as_str().unwrap_or("null")
    );
}

pub fn print_item_list(json: &Value, indent_level: usize) -> String {
    json.as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|item| print_track_episode(item, indent_level))
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn print_item_list_pretty(json: &Value, indent_level: usize) -> String {
    json.as_array()
        .map(|arr| {
            arr.iter()
                .map(|item| print_track_episode_pretty(item, indent_level))
                .collect::<Vec<String>>()
                .join("\n")
        })
        .unwrap_or(format!("{}{}", "\t".repeat(indent_level), "None"))
}

pub fn print_device_pretty(json: &Value, indent_level: usize) -> String {
    return format!(
        "{}{} ({})",
        "\t".repeat(indent_level),
        json["name"].as_str().unwrap_or("null"),
        json["id"].as_str().unwrap_or("null")
    );
}

pub fn print_device(json: &Value, indent_level: usize) -> String {
    return format!(
        "{}{}",
        "\t".repeat(indent_level),
        json["id"].as_str().unwrap_or("null")
    );
}

pub fn print_device_list(json: &Value, indent_level: usize) -> String {
    json.as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|item| print_device(item, indent_level))
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn print_device_list_pretty(json: &Value, indent_level: usize) -> String {
    json.as_array()
        .map(|arr| {
            arr.iter()
                .map(|item| print_device_pretty(item, indent_level))
                .collect::<Vec<String>>()
                .join("\n")
        })
        .unwrap_or(format!("{}{}", "\t".repeat(indent_level), "None"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn track() -> Value {
        return json!({
            "type": "track", "uri": "spotify:track:t", "name": "Song",
            "album": {"name": "Album"}, "artists": [{"name": "Artist"}]
        });
    }

    fn episode() -> Value {
        return json!({
            "type": "episode", "uri": "spotify:episode:e", "name": "Ep",
            "show": {"name": "Show", "publisher": "Pub"}
        });
    }

    #[test]
    fn items() {
        assert_eq!(
            print_track_episode_pretty(&track(), 1),
            "\t\"Song\" - Album by Artist"
        );
        assert_eq!(
            print_track_episode_pretty(&episode(), 0),
            "\"Ep\" - Show by Pub"
        );
        assert_eq!(print_track_episode_pretty(&json!({}), 0), "Unknown Item");
        assert_eq!(print_track_episode(&track(), 1), "\tspotify:track:t");
        assert_eq!(print_track_episode(&json!({}), 0), "null");
    }

    #[test]
    fn item_lists() {
        let list = json!([track(), episode()]);
        assert_eq!(
            print_item_list(&list, 0),
            "spotify:track:t\nspotify:episode:e"
        );
        assert_eq!(
            print_item_list_pretty(&list, 0),
            "\"Song\" - Album by Artist\n\"Ep\" - Show by Pub"
        );
        assert_eq!(print_item_list(&json!({}), 0), "");
        assert_eq!(print_item_list_pretty(&json!({}), 1), "\tNone");
    }

    #[test]
    fn device_lists() {
        let list = json!([{"name": "Phone", "id": "p1"}, {"name": "Laptop", "id": "l1"}]);
        assert_eq!(print_device_list(&list, 0), "p1\nl1");
        assert_eq!(
            print_device_list_pretty(&list, 1),
            "\tPhone (p1)\n\tLaptop (l1)"
        );
        assert_eq!(print_device_list_pretty(&json!({}), 0), "None");
    }
}
