//! Album grouping for the offline library, independent of the playback queue.
use crate::subsonic::models::Child;

#[derive(Debug, Clone)]
pub struct OfflineAlbum {
    pub name: String,
    pub artist: String,
    pub songs: Vec<usize>,
}

pub fn albums(songs: &[Child]) -> Vec<OfflineAlbum> {
    let mut groups: std::collections::BTreeMap<String, OfflineAlbum> = Default::default();
    for (index, song) in songs.iter().enumerate() {
        let artist = song.artist.as_deref().unwrap_or("Unknown artist");
        let name = song.album.as_deref().unwrap_or("Unknown album");
        let key = match song.album_id.as_ref().or(song.parent.as_ref()) {
            Some(id) if !id.is_empty() => format!("id:{id}"),
            _ => format!("name:{artist}\0{name}"),
        };
        groups.entry(key).or_insert_with(|| OfflineAlbum {
            name: name.to_string(), artist: artist.to_string(), songs: Vec::new(),
        }).songs.push(index);
    }
    let mut result: Vec<_> = groups.into_values().collect();
    for album in &mut result {
        album.songs.sort_by_key(|&i| (songs[i].disc_number.unwrap_or(0), songs[i].track.unwrap_or(0), songs[i].title.clone()));
    }
    result.sort_by(|a, b| (&a.artist, &a.name).cmp(&(&b.artist, &b.name)));
    result
}

/// Each row maps to an album and an optional position in that album.
pub fn rows(albums: &[OfflineAlbum]) -> Vec<(usize, Option<usize>)> {
    let mut rows = Vec::new();
    for (i, album) in albums.iter().enumerate() {
        rows.push((i, None));
        rows.extend((0..album.songs.len()).map(|j| (i, Some(j))));
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn groups_album_by_parent_and_orders_tracks() {
        let songs: Vec<Child> = serde_json::from_str(r#"[
          {"id":"2","parent":"album1","title":"second","artist":"Artist","album":"Record","track":2},
          {"id":"1","parent":"album1","title":"first","artist":"Guest","album":"Record","track":1},
          {"id":"3","parent":"album2","title":"other","artist":"Artist","album":"Record","track":1}
        ]"#).unwrap();
        let albums = albums(&songs);
        assert_eq!(albums.len(), 2);
        let record = albums.iter().find(|a| a.songs.len() == 2).unwrap();
        assert_eq!(record.songs, vec![1, 0]);
        assert_eq!(rows(&albums).len(), 5);
    }
}
