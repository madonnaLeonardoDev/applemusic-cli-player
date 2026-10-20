let res = MusicKit.getInstance().queue._queueItems;
let result = [];
res.forEach(e => {
    result.push({
        id: e.item.id,
        title: e.item.attributes.name,
        artist: e.item.attributes.artistName,
        album: e.item.attributes.albumName,
        duration_ms: e.item.attributes.durationInMillis
    })
    
});



