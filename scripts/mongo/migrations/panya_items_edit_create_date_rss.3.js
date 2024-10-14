const db = connect("mongodb://localhost:27017/panya");




// Perform operations on the collection
const channel_ids = db.channels.find({
    "source_type": "rss_feed"
})
    .map(doc => doc.id);
if (channel_ids) {
    print("modified:", db.items.updateMany(
        {
            channel_id: { $nin: channel_ids }
        },
        [
            {
                $set: {
                    create_date: {
                        $cond: {
                            if: { $lt: ["$create_date", { $divide: [{ $toLong: new Date() }, 100] }] },
                            then: { $toLong: { $multiply: ["$create_date", 1000] } },
                            else: "$create_date"
                        }
                    }
                }
            }
        ]
    ).modifiedCount);
} else {
    print("Nothing to migrate in panya_items_edit_create_date_rss.3.");
}

print("Migration panya_items_edit_create_date_rss.3 completed successfully.");
