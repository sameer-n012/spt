#[derive(Debug)]
pub enum UriType {
    Track,
    Album,
    Artist,
    Playlist,
    Episode,
    Show,
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
    } else if uri.starts_with("spotify:episode") {
        return UriType::Episode;
    } else if uri.starts_with("spotify:show") {
        return UriType::Show;
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
        UriType::Episode => return Some(format!("spotify:episode:{}", id)),
        UriType::Show => return Some(format!("spotify:show:{}", id)),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_type() {
        assert!(matches!(get_uri_type("spotify:track:1"), UriType::Track));
        assert!(matches!(get_uri_type("spotify:album:1"), UriType::Album));
        assert!(matches!(get_uri_type("spotify:artist:1"), UriType::Artist));
        assert!(matches!(
            get_uri_type("spotify:playlist:1"),
            UriType::Playlist
        ));
        assert!(matches!(
            get_uri_type("spotify:episode:1"),
            UriType::Episode
        ));
        assert!(matches!(get_uri_type("spotify:show:1"), UriType::Show));
        assert!(matches!(get_uri_type("track:1"), UriType::Unknown));
    }

    #[test]
    fn id_uri_url() {
        assert_eq!(
            get_id_from_uri("spotify:track:abc"),
            Some("abc".to_string())
        );
        assert_eq!(get_id_from_uri("spotify:track"), None);
        for t in [UriType::Track, UriType::Episode, UriType::Show] {
            let uri = get_uri_from_id(&t, "abc").unwrap();
            assert_eq!(get_id_from_uri(&uri), Some("abc".to_string()));
        }
        assert_eq!(get_uri_from_id(&UriType::Unknown, "abc"), None);
        assert_eq!(
            get_url_from_uri("spotify:album:abc"),
            Some("https://open.spotify.com/album/abc".to_string())
        );
    }

    #[test]
    fn split() {
        assert_eq!(split_uris(" a, b ,c "), vec!["a", "b", "c"]);
    }
}
