// general stuff
pub const PRAGMA_FOREIGN_KEY_ENFORCE: &str =
    "PRAGMA foreign_keys=ON";

// create statements
pub const CREATE_FILES_TABLE: &str =
    "CREATE TABLE IF NOT EXISTS files (
            filename TEXT PRIMARY KEY,
            file_source TEXT NOT NULL,
            res_width INT NOT NULL,
            res_height INT NOT NULL
    )";

pub const CREATE_TAGS_TABLE: &str =
    "CREATE TABLE IF NOT EXISTS tags (
            tag TEXT PRIMARY KEY,
            tag_type TEXT NOT NULL
    )";

pub const CREATE_FILE_TAGS_TABLE: &str =
    "CREATE TABLE IF NOT EXISTS file_tags (
            filename TEXT NOT NULL,
            tag TEXT NOT NULL,
            PRIMARY KEY(filename, tag),
            FOREIGN KEY(filename) REFERENCES files(filename),
            FOREIGN KEY(tag) REFERENCES tags(tag)
    )";

// select statements
pub const SELECT_FILE: &str =
    "SELECT filename, file_source, res_width, res_height FROM files
    WHERE filename = (?)";

pub const SELECT_ALL_FILES: &str =
    "SELECT filename, file_source, res_width, res_height FROM files
    ORDER BY filename ASC";

pub const SELECT_FILES_FOR_TAGS_TAG_COUNT: &str= "TAG_COUNT";
pub const SELECT_FILES_FOR_INCLUDING_TAGS_PLACEHOLDER: &str = "INCLUDE_TAG_LIST";
pub const SELECT_FILES_FOR_INCLUDING_TAGS: &str =
    "SELECT DISTINCT filename, file_source, res_width, res_height
    FROM files
    WHERE filename IN (
        SELECT filename FROM file_tags
        WHERE tag IN (INCLUDE_TAG_LIST)
        GROUP BY filename
        HAVING COUNT(DISTINCT tag)=TAG_COUNT
    )
    ORDER BY filename ASC";

pub const SELECT_FILES_FOR_EXCLUDING_TAGS_PLACEHOLDER: &str = "EXCLUDE_TAG_LIST";
pub const SELECT_FILES_FOR_EXCLUDING_TAGS: &str =
    "SELECT DISTINCT filename, file_source, res_width, res_height
    FROM files
    WHERE filename NOT IN (
        SELECT filename
        FROM file_tags
        WHERE tag IN (EXCLUDE_TAG_LIST)
        GROUP BY filename
        HAVING COUNT(DISTINCT tag)>0
    )
    ORDER BY filename ASC";

pub const SELECT_FILES_FOR_INCLUDING_AND_EXCLUDING_TAGS: &str =
    "SELECT DISTINCT filename, file_source, res_width, res_height
    FROM files
    WHERE filename IN (
        SELECT filename FROM file_tags
        WHERE tag IN (INCLUDE_TAG_LIST)
        GROUP BY filename
        HAVING COUNT(DISTINCT tag)=TAG_COUNT
    )
    AND filename NOT IN (
        SELECT filename
        FROM file_tags
        WHERE tag IN (EXCLUDE_TAG_LIST)
        GROUP BY filename
        HAVING COUNT(DISTINCT tag)>0
    )
    ORDER BY filename ASC";

pub const SELECT_TAGS_FOR_FILE: &str =
    "SELECT tags.tag, tags.tag_type FROM file_tags
    JOIN tags ON file_tags.tag = tags.tag
    WHERE file_tags.filename = (?)
    ORDER BY tags.tag_type ASC, tags.tag ASC";

pub const SELECT_ALL_TAGS: &str =
    "SELECT tag, tag_type FROM tags
    ORDER BY tag_type ASC, tag ASC";

pub const SELECT_TAGS_WITH_TYPE_PLACEHOLDER: &str = "TAG_TYPE_LIST";
pub const SELECT_TAGS_WITH_TYPE: &str =
    "SELECT tag, tag_type FROM tags
    WHERE tag_type IN (TAG_TYPE_LIST)
    ORDER BY tag_type ASC, tag ASC";

// insert statements
pub const INSERT_TAG_INTO_TAG_LIST: &str =
    "INSERT OR IGNORE INTO tags (tag, tag_type) VALUES (?, ?)";

pub const INSERT_FILE_INTO_FILE_LIST: &str =
    "INSERT OR IGNORE INTO files (filename, file_source, res_width, res_height) VALUES (?, ?, ?, ?)";

pub const INSERT_TAG_FOR_FILE: &str =
    "INSERT OR IGNORE INTO file_tags (filename, tag) VALUES (?, ?)";

// delete statements
pub const DELETE_UNUSED_TAGS: &str =
    "DELETE FROM tags
    WHERE tag IN
        (SELECT tags.tag FROM tags
        LEFT JOIN file_tags ON tags.tag=file_tags.tag
        WHERE file_tags.tag IS NULL
        )";

pub const DELETE_FILE_FROM_FILE_LIST: &str =
    "DELETE FROM files WHERE filename=(?)";

pub const DELETE_TAG_FROM_FILE: &str =
    "DELETE FROM file_tags WHERE filename=(?) AND tag=(?)";

pub const DELETE_ALL_TAGS_FROM_FILE: &str =
    "DELETE FROM file_tags WHERE filename=(?)";

// update statements
pub const UPDATE_FILE_SOURCE: &str =
    "UPDATE files
    SET file_source = (?)
    WHERE filename = (?)";

// clear tables
pub const CLEAR_FILE_LIST: &str =
    "DELETE FROM files";
pub const CLEAR_FILE_TAGS: &str =
    "DELETE FROM file_tags";
pub const CLEAR_TAG_LIST: &str =
    "DELETE FROM tags";
