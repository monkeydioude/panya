function cleanUrl(url) {
    // Remove "http://" or "https://" if present
    if (url.startsWith('http://')) {
        url = url.slice(7);
    } else if (url.startsWith('https://')) {
        url = url.slice(8);
    }

    // Remove any trailing slashes
    url = url.replace(/\/+$/, '');

    // Extract the hostname (domain) and pathname (path)
    const pathStartIndex = url.indexOf("/");

    let host;
    let path = '';

    if (pathStartIndex !== -1) {
        host = url.substring(0, pathStartIndex);
        path = url.substring(pathStartIndex);
    } else {
        host = url;
    }

    // Remove "www" or "www3" if present
    if (host.startsWith('www')) {
        const parts = host.split('.');
        if (parts.length > 2) {
            host = parts.slice(1).join('.');
        }
    }

    // Combine the cleaned host with the path
    const cleanedUrl = path ? `${host}${path}` : host;

    return cleanedUrl;
}

const db = connect("mongodb://localhost:27017/panya");

// Perform operations on the collection
db.channels.find({}).forEach(function (doc) {
    doc.name = cleanUrl(doc.name);
    db.channels.save(doc);
});