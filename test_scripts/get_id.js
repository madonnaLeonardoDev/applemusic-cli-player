let mediaItem = MusicKit.getInstance().nowPlayingItem;
    
    if (mediaItem) {
        console.log({
            id: id,title: mediaItem.attributes.name,
            artist: mediaItem.attributes.artistName,
            album: mediaItem.attributes.albumName,
            duration_ms: mediaItem.attributes.durationInMillis,
            has_lyrics: mediaItem.attributes.hasLyrics,
            artwork: mediaItem.attributes.artwork });
    }