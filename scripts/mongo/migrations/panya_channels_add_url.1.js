const db = connect("mongodb://localhost:27017/panya");

// Perform operations on the collection
db.channels.find({}).forEach(function (doc) {
    doc.url = doc.name.startsWith("http") ? doc.name : `https://${doc.name}`;

    // Save the updated document back to the collection
    db.channels.save(doc);
});

print("Migration panya_channels_add_url.1 completed successfully.");
