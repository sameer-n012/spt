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
        .and_then(|arr| {
            Some(
                arr.iter()
                    .map(|item| print_track_episode_pretty(item, indent_level))
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
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
        .and_then(|arr| {
            Some(
                arr.iter()
                    .map(|item| print_device_pretty(item, indent_level))
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
        })
        .unwrap_or(format!("{}{}", "\t".repeat(indent_level), "None"))
}
