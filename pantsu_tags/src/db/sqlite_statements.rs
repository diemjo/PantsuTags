// create statements
pub const DB_INIT_TABLES: &str =
    "CREATE TABLE IF NOT EXISTS images (
            filename TEXT PRIMARY KEY,
            image_source_type TEXT NOT NULL,
            image_source TEXT,
            res_width INT NOT NULL,
            res_height INT NOT NULL,
            date_added TEXT NOT NULL,
            date_modified TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS tags (
            tag TEXT NOT NULL,
            tag_type TEXT NOT NULL,
            PRIMARY KEY(tag, tag_type)
    );
    CREATE TABLE IF NOT EXISTS image_tags (
            filename TEXT NOT NULL,
            tag TEXT NOT NULL,
            tag_type TEXT NOT NULL,
            tag_author TEXT NOT NULL,
            date_added TEXT NOT NULL,
            PRIMARY KEY(filename, tag),
            FOREIGN KEY(filename) REFERENCES images(filename),
            FOREIGN KEY(tag, tag_type) REFERENCES tags(tag, tag_type)
    );";

// select statements
pub const SELECT_IMAGE: &str =
    "SELECT filename, image_source_type, image_source, res_width, res_height, date_added, date_modified
    FROM images
    WHERE filename = (?)";

pub const SELECT_IMAGES_SORT_BY: &str = "SORT_ORDER";
pub const SAUCE_TYPE_PLACEHOLDER: &str = "SAUCE_TYPE";
pub const SELECT_ALL_IMAGES: &str =
    "SELECT filename, image_source_type, image_source, res_width, res_height, date_added, date_modified
    FROM images
    WHERE image_source_type LIKE 'SAUCE_TYPE'
    ORDER BY SORT_ORDER";

pub const SELECT_IMAGES_FOR_TAGS_TAG_COUNT: &str= "TAG_COUNT";
pub const SELECT_IMAGES_FOR_INCLUDING_TAGS_PLACEHOLDER: &str = "INCLUDE_TAG_LIST";
pub const SELECT_IMAGES_FOR_INCLUDING_TAGS: &str =
    "SELECT DISTINCT filename, image_source_type, image_source, res_width, res_height, date_added, date_modified
    FROM images
    WHERE filename IN (
        SELECT filename
        FROM image_tags
        WHERE tag_type || ':' || tag IN (INCLUDE_TAG_LIST)
        GROUP BY filename
        HAVING COUNT(tag)=TAG_COUNT
    )
    AND image_source_type LIKE 'SAUCE_TYPE'
    ORDER BY SORT_ORDER";

pub const SELECT_IMAGES_FOR_EXCLUDING_TAGS_PLACEHOLDER: &str = "EXCLUDE_TAG_LIST";
pub const SELECT_IMAGES_FOR_EXCLUDING_TAGS: &str =
    "SELECT DISTINCT filename, image_source_type, image_source, res_width, res_height, date_added, date_modified
    FROM images
    WHERE filename NOT IN (
        SELECT filename
        FROM image_tags
        WHERE tag_type || ':' || tag IN (EXCLUDE_TAG_LIST)
        GROUP BY filename
        HAVING COUNT(tag)>0
    )
    AND image_source_type LIKE 'SAUCE_TYPE'
    ORDER BY SORT_ORDER";

pub const SELECT_IMAGES_FOR_INCLUDING_AND_EXCLUDING_TAGS: &str =
    "SELECT DISTINCT filename, image_source_type, image_source, res_width, res_height, date_added, date_modified
    FROM images
    WHERE filename IN (
        SELECT filename
        FROM image_tags
        WHERE tag_type || ':' || tag IN (INCLUDE_TAG_LIST)
        GROUP BY filename
        HAVING COUNT(tag)=TAG_COUNT
    )
    AND filename NOT IN (
        SELECT filename
        FROM image_tags
        WHERE tag_type || ':' || tag IN (EXCLUDE_TAG_LIST)
        GROUP BY filename
        HAVING COUNT(tag)>0
    )
    AND image_source_type LIKE 'SAUCE_TYPE'
    ORDER BY SORT_ORDER";

pub const SELECT_TAGS_SORT_BY: &str = "SORT_ORDER";
pub const SELECT_TAGS_FOR_IMAGE: &str =
    "SELECT tags.tag, tags.tag_type, image_tags.tag_author, image_tags.date_added
    FROM image_tags
    JOIN tags ON image_tags.tag = tags.tag
    WHERE image_tags.filename = (?)
    ORDER BY SORT_ORDER";

pub const SELECT_ALL_TAGS: &str =
    "SELECT tag, tag_type
    FROM tags
    ORDER BY tag_type ASC, tag ASC";

pub const SELECT_TAGS_WITH_TYPE_PLACEHOLDER: &str = "TAG_TYPE_LIST";
pub const SELECT_TAGS_WITH_TYPE: &str =
    "SELECT tag, tag_type
    FROM tags
    WHERE tag_type IN (TAG_TYPE_LIST)
    ORDER BY tag_type ASC, tag ASC";

pub const SELECT_TAGS_FOR_IMAGE_WITH_TYPE: &str =
    "SELECT tags.tag, tags.tag_type, image_tags.tag_author, image_tags.date_added
    FROM image_tags
    JOIN tags ON image_tags.tag = tags.tag
    WHERE image_tags.filename = (?)
    AND tags.tag_type IN (TAG_TYPE_LIST)
    ORDER BY SORT_ORDER";

// insert statements
pub const INSERT_TAG_INTO_TAG_LIST: &str =
    "INSERT OR IGNORE INTO tags (tag, tag_type) VALUES (?, ?)";

pub const INSERT_IMAGE_INTO_IMAGES: &str =
    "INSERT INTO images (filename, image_source_type, image_source, res_width, res_height, date_added, date_modified) VALUES (?, ?, ?, ?, ?, ?, ?)";

pub const INSERT_TAG_FOR_IMAGE: &str =
    "INSERT OR IGNORE INTO image_tags (filename, tag, tag_type, tag_author, date_added) VALUES (?, ?, ?, ?, ?)";

// delete statements
pub const DELETE_UNUSED_TAGS: &str =
    "DELETE FROM tags
    WHERE tag IN (
        SELECT tags.tag
        FROM tags
        LEFT JOIN image_tags ON tags.tag=image_tags.tag
        WHERE image_tags.tag IS NULL
    )";

pub const DELETE_IMAGE_FROM_IMAGES: &str =
    "DELETE FROM images WHERE filename=(?)";

pub const DELETE_TAG_FROM_IMAGES: &str =
    "DELETE FROM image_tags WHERE filename=(?) AND tag_type=(?) AND tag=(?)";

pub const DELETE_ALL_TAGS_FROM_IMAGE: &str =
    "DELETE FROM image_tags WHERE filename=(?)";

// update statements
pub const UPDATE_IMAGE_SOURCE: &str =
    "UPDATE images
    SET image_source_type = (?),
        image_source = (?)
    WHERE filename = (?)";

pub const UPDATE_IMAGE_DATE_MODIFIED: &str =
    "Update images
    SET date_modified = (?)
    WHERE filename = (?)";

// clear tables
pub const CLEAR_IMAGES: &str =
    "DELETE FROM images";
pub const CLEAR_IMAGE_TAGS: &str =
    "DELETE FROM image_tags";
pub const CLEAR_TAGS: &str =
    "DELETE FROM tags";
