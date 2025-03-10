#[derive(Debug)]
pub enum UriType {
    Track,
    Album,
    Artist,
    Playlist,
    Unknown,
}

pub fn get_uri_type(uri: &str) -> UriType {
    if uri.starts_with("spotify:track") {
        return UriType::Track;
    } else if uri.starts_with("spotify:album") {
        return UriType::Album;
    } else if uri.starts_with("spotify:artist") {
        return UriType::Artist;
    } else if uri.starts_with("spotify:playlist") {
        return UriType::Playlist;
    } else {
        return UriType::Unknown;
    }
}

pub fn get_id_from_uri(uri: &str) -> Option<String> {
    let uri_parts: Vec<&str> = uri.split(':').collect();
    if uri_parts.len() < 3 {
        return None;
    }
    return Some(uri_parts[2].to_string());
}

pub fn get_uri_from_id(uri_type: &UriType, id: &str) -> Option<String> {
    match uri_type {
        UriType::Track => return Some(format!("spotify:track:{}", id)),
        UriType::Album => return Some(format!("spotify:album:{}", id)),
        UriType::Artist => return Some(format!("spotify:artist:{}", id)),
        UriType::Playlist => return Some(format!("spotify:playlist:{}", id)),
        _ => return None,
    }
}

pub fn get_url_from_uri(uri: &str) -> Option<String> {
    let uri_parts: Vec<&str> = uri.split(':').collect();
    if uri_parts.len() < 3 {
        return None;
    }
    return Some(format!(
        "https://open.spotify.com/{}/{}",
        uri_parts[1], uri_parts[2]
    ));
}

pub fn split_uris(uris: &str) -> Vec<String> {
    return uris
        .trim()
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
}
