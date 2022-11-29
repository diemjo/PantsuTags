use chrono::{Utc};
use rusqlite::{Connection, ffi, params, Transaction};
use crate::common::error::Result;
use crate::common::image_handle::ImageHandle;
use crate::common::image_info::{ImageInfo, DATE_TIME_FORMAT};
use crate::common::pantsu_tag::{PantsuTag, PantsuTagType, PantsuTagAuthor, PantsuTagInfo};
use crate::db::{SauceType, sqlite_statements};
use crate::{Error, Sauce, sauce};

pub(crate) fn db_version(connection: &Connection) -> Result<usize> {
    Ok(connection.pragma_query_value(None, "user_version", |r| r.get(0))?)
}

// UPDATE MODIFICATION DATE
pub(crate) fn modify_image(transaction: &Transaction, image: &ImageHandle) -> Result<()> {
    let now = Utc::now().naive_utc().format(DATE_TIME_FORMAT).to_string();
    let mut update_image_date_modified_stmt = transaction.prepare(sqlite_statements::UPDATE_IMAGE_DATE_MODIFIED)?;
    update_image_date_modified_stmt.execute(params![&now, image.get_filename()])?;
    Ok(())
}

// INSERT
pub(crate) fn add_tags_to_tag_list(transaction: &Transaction, tags: &Vec<&PantsuTag>) -> Result<()> {
    let mut add_tag_list_stmt = transaction.prepare(sqlite_statements::INSERT_TAG_INTO_TAG_LIST)?;
    for &tag in tags {
        add_tag_list_stmt.execute([&tag.tag_name, &tag.tag_type.serialize()])?;
    }
    Ok(())
}

pub(crate) fn add_image_to_images(transaction: &Transaction, image: &ImageHandle, res: (u32, u32)) -> Result<()> {
    let mut add_image_stmt = transaction.prepare(sqlite_statements::INSERT_IMAGE_INTO_IMAGES)?;
    let now = Utc::now().naive_utc().format(DATE_TIME_FORMAT).to_string();
    let res = add_image_stmt.execute(params![image.get_filename(), sauce::NOT_CHECKED_FLAG, None as Option<String>, res.0, res.1, &now, &now]);
    // check for primary key constraint
    return if let Err(rusqlite::Error::SqliteFailure(ffi::Error { code: _, extended_code: 1555 }, ..)) = res {
        Err(Error::SQLPrimaryKeyError(res.unwrap_err()))
    } else if let Err(e) = res {
        Err(Error::SQLError(e))
    } else {
        Ok(())
    }
}

pub(crate) fn add_tags_to_image(transaction: &Transaction, image: &ImageHandle, tags: &Vec<&PantsuTag>, tag_author: &PantsuTagAuthor) -> Result<()> {
    let mut add_tag_stmt = transaction.prepare(sqlite_statements::INSERT_TAG_FOR_IMAGE)?;
    let now = Utc::now().naive_utc().format(DATE_TIME_FORMAT).to_string();
    let tag_author = tag_author.serialize();
    for &tag in tags {
        add_tag_stmt.execute(params![image.get_filename(), tag.tag_name, tag.tag_type.serialize(), &tag_author, &now])?;
    }
    Ok(())
}

// UPDATE
pub(crate) fn update_image_source(transaction: &Transaction, image: &ImageHandle, sauce: &Sauce) -> Result<()> {
    let mut update_image_stmt = transaction.prepare(sqlite_statements::UPDATE_IMAGE_SOURCE)?;
    update_image_stmt.execute(params![sauce.get_type(), sauce.get_value(), image.get_filename()])?;
    Ok(())
}

// DELETE
pub(crate) fn remove_unused_tags(transaction: &Transaction) -> Result<()> {
    transaction.execute(sqlite_statements::DELETE_UNUSED_TAGS, [])?;
    Ok(())
}

pub(crate) fn remove_image_from_images(transaction: &Transaction, image: &ImageHandle) -> Result<()> {
    let mut remove_image_stmt = transaction.prepare(sqlite_statements::DELETE_IMAGE_FROM_IMAGES)?;
    remove_image_stmt.execute([image.get_filename()])?;
    Ok(())
}

pub(crate) fn remove_tags_from_images(transaction: &Transaction, image: &ImageHandle, tags: &Vec<&PantsuTag>) -> Result<()> {
    let mut remove_tag_stmt = transaction.prepare(sqlite_statements::DELETE_TAG_FROM_IMAGES)?;
    for &tag in tags {
        remove_tag_stmt.execute(params![image.get_filename(), tag.tag_type.serialize(), tag.tag_name])?;
    }
    Ok(())
}

pub(crate) fn remove_all_tags_from_image(transaction: &Transaction, image: &ImageHandle) -> Result<()> {
    let mut remove_tags_stmt = transaction.prepare(sqlite_statements::DELETE_ALL_TAGS_FROM_IMAGE)?;
    remove_tags_stmt.execute([image.get_filename()])?;
    Ok(())
}

pub(crate) fn clear_all_image_tags(transaction: &Transaction) -> Result<()> {
    transaction.execute(sqlite_statements::CLEAR_IMAGE_TAGS, [])?;
    Ok(())
}

pub(crate) fn clear_all_images(transaction: &Transaction) -> Result<()> {
    transaction.execute(sqlite_statements::CLEAR_IMAGES, [])?;
    Ok(())
}

pub(crate) fn clear_all_tags(transaction: &Transaction) -> Result<()> {
    transaction.execute(sqlite_statements::CLEAR_TAGS, [])?;
    Ok(())
}

// SELECT
pub(crate) fn get_image(connection: &Connection, image: &ImageHandle) -> Result<Option<ImageInfo>> {
    let mut stmt = connection.prepare(sqlite_statements::SELECT_IMAGE)?;
    let rows = stmt.query([image.get_filename()])?;
    query_helpers::query_row_as_image(rows)
}
/*
pub fn get_all_files(connection: &Connection) -> Result<Vec<ImageHandle>> {
    let mut stmt = connection.prepare(sqlite_statements::SELECT_ALL_FILES)?;
    query_helpers::query_rows_as_files(&mut stmt, [])
}
*/
pub(crate) fn get_images(connection: &Connection, included_tags: &Vec<PantsuTag>, excluded_tags: &Vec<PantsuTag>, sauce_type: SauceType) -> Result<Vec<ImageInfo>> {
    let formatted_stmt =
        if included_tags.len()!=0 && excluded_tags.len()!=0 {
            sqlite_statements::SELECT_IMAGES_FOR_INCLUDING_AND_EXCLUDING_TAGS
                .replace(sqlite_statements::SELECT_IMAGES_FOR_INCLUDING_TAGS_PLACEHOLDER, &query_helpers::repeat_vars(included_tags.len()))
                .replace(sqlite_statements::SELECT_IMAGES_FOR_EXCLUDING_TAGS_PLACEHOLDER, &query_helpers::repeat_vars(excluded_tags.len()))
                .replace(sqlite_statements::SELECT_IMAGES_FOR_TAGS_TAG_COUNT, &included_tags.len().to_string())
        } else if excluded_tags.len()!=0 {
            sqlite_statements::SELECT_IMAGES_FOR_EXCLUDING_TAGS
                .replace(sqlite_statements::SELECT_IMAGES_FOR_EXCLUDING_TAGS_PLACEHOLDER, &query_helpers::repeat_vars(excluded_tags.len()))
        } else if included_tags.len()!=0 {
            sqlite_statements::SELECT_IMAGES_FOR_INCLUDING_TAGS
                .replace(sqlite_statements::SELECT_IMAGES_FOR_INCLUDING_TAGS_PLACEHOLDER, &query_helpers::repeat_vars(included_tags.len()))
                .replace(sqlite_statements::SELECT_IMAGES_FOR_TAGS_TAG_COUNT, &included_tags.len().to_string())
        } else {
            sqlite_statements::SELECT_ALL_IMAGES.to_string()
        };
    let formatted_stmt = formatted_stmt.replace(sqlite_statements::SAUCE_TYPE_PLACEHOLDER, match sauce_type {
        SauceType::Existing => sauce::EXISTING_FLAG,
        SauceType::NotExisting => sauce::NOT_EXISTING_FLAG,
        SauceType::NotChecked => sauce::NOT_CHECKED_FLAG,
        SauceType::Any => "%",
    });
    let mut stmt = connection.prepare(&formatted_stmt)?;

    let included_tags: Vec<String> = included_tags.iter().map(|t| t.serialize()).collect();
    let excluded_tags: Vec<String> = excluded_tags.iter().map(|t| t.serialize()).collect();
    if included_tags.len()!=0 && excluded_tags.len()!=0 {
        let mut vec = Vec::<String>::new();
        vec.extend(included_tags);
        vec.extend(excluded_tags);
        let params = rusqlite::params_from_iter(vec.iter());
        query_helpers::query_rows_as_images(stmt.query(params)?)
    } else if excluded_tags.len()!=0 {
        let params = rusqlite::params_from_iter(excluded_tags.iter());
        query_helpers::query_rows_as_images(stmt.query(params)?)
    } else if included_tags.len()!=0 {
        let params = rusqlite::params_from_iter(included_tags.iter());
        query_helpers::query_rows_as_images(stmt.query(params)?)
    } else {
        query_helpers::query_rows_as_images(stmt.query([])?)
    }
}

pub(crate) fn get_tags_for_image(connection: &Connection, image: &ImageHandle) -> Result<Vec<PantsuTagInfo>> {
    let mut stmt = connection.prepare(sqlite_statements::SELECT_TAGS_FOR_IMAGE)?;
    let rows = stmt.query([image.get_filename()])?;
    query_helpers::query_rows_as_tag_infos(rows)
}

pub(crate) fn get_all_tags(connection: &Connection) -> Result<Vec<PantsuTag>> {
    let mut stmt = connection.prepare(sqlite_statements::SELECT_ALL_TAGS)?;
    let rows = stmt.query([])?;
    query_helpers::query_rows_as_tags(rows)
}

pub(crate) fn get_tags_with_types(connection: &Connection, types: &Vec<PantsuTagType>) -> Result<Vec<PantsuTag>> {
    let formatted_stmt = sqlite_statements::SELECT_TAGS_WITH_TYPE
        .replace(sqlite_statements::SELECT_TAGS_WITH_TYPE_PLACEHOLDER, &query_helpers::repeat_vars(types.len()));
    let mut stmt = connection.prepare(&formatted_stmt)?;

    let params = rusqlite::params_from_iter(types.iter().map(|t| t.serialize()).into_iter());
    let rows = stmt.query(params)?;
    query_helpers::query_rows_as_tags(rows)
}

pub(crate) fn get_tags_for_image_with_types(connection: &Connection, image: &ImageHandle, types: &Vec<PantsuTagType>) -> Result<Vec<PantsuTagInfo>> {
    let formatted_stmt = sqlite_statements::SELECT_TAGS_FOR_IMAGE_WITH_TYPE
        .replace(sqlite_statements::SELECT_TAGS_WITH_TYPE_PLACEHOLDER, &query_helpers::repeat_vars(types.len()));
    let mut stmt = connection.prepare(&formatted_stmt)?;

    let mut vec = vec![image.get_filename().to_string()];
    vec.extend(types.iter().map(|t| t.serialize()));
    let params = rusqlite::params_from_iter(vec);
    let rows = stmt.query(params)?;
    query_helpers::query_rows_as_tag_infos(rows)
}

mod query_helpers {

    use chrono::NaiveDateTime;
    use rusqlite::{Row, Rows};
    use crate::common::error::Result;
    use crate::common::image_info::{ImageInfo, DATE_TIME_FORMAT};
    use crate::sauce::{EXISTING_FLAG, NOT_EXISTING_FLAG, NOT_CHECKED_FLAG};
    use crate::common::pantsu_tag::{PantsuTag, PantsuTagInfo, PantsuTagAuthor};
    use crate::{Error, Sauce, PantsuTagType, sauce, ImageHandle};

    pub fn query_row_as_image(rows: Rows) -> Result<Option<ImageInfo>> {
        let rows = query_rows_as_images(rows)?;
        match rows.into_iter().next() {
            Some(i) => Ok(Some(i)),
            None => Ok(None)
        }
    }

    pub fn query_rows_as_images(rows: Rows) -> Result<Vec<ImageInfo>> {
        rows.and_then(image_info_from_row).collect::<Result<Vec<ImageInfo>>>()
    }

    fn image_info_from_row(row: &Row) -> Result<ImageInfo> {
        Ok(
            ImageInfo::new(
                ImageHandle::new(row.get(0)?)?,
                match row.get::<usize, String>(1)?.as_str() {
                    EXISTING_FLAG => Sauce::Match(sauce::url_from_str(&row.get::<usize, String>(2)?)?),
                    NOT_EXISTING_FLAG => Sauce::NotExisting,
                    NOT_CHECKED_FLAG => Sauce::NotChecked,
                    s => return Err(Error::InvalidSauceType(s.to_string()))
                },
                (row.get(3)?, row.get(4)?),
                NaiveDateTime::parse_from_str(row.get::<usize, String>(5)?.as_str(), DATE_TIME_FORMAT)
                    .or_else(|e| Err(Error::InvalidDateFormat(e)))?,
                NaiveDateTime::parse_from_str(row.get::<usize, String>(6)?.as_str(), DATE_TIME_FORMAT)
                    .or_else(|e| Err(Error::InvalidDateFormat(e)))?
            )
        )
    }

    pub fn query_rows_as_tags(rows: Rows) -> Result<Vec<PantsuTag>> {
        let rows: Vec<PantsuTag> = rows
            .mapped(|row| -> rusqlite::Result<(String, String)> {
                Ok((row.get(0)?, row.get(1)?))
            })
            .map(|r| {
                match r {
                    Ok((tag_name, tag_type)) => Ok(PantsuTag { tag_type: PantsuTagType::deserialize(&tag_type)? , tag_name: tag_name }),
                    Err(e) => Err(Error::SQLError(e))
                }
            })
            .collect::<Result<Vec<PantsuTag>>>()?;
        Ok(rows)
    }

    pub fn query_rows_as_tag_infos(rows: Rows) -> Result<Vec<PantsuTagInfo>> {
        let rows: Vec<PantsuTagInfo> = rows
            .mapped(|row| -> rusqlite::Result<(String, String, String, String)> {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .map(|r| {
                match r {
                    Ok((tag_name, tag_type, tag_author, date_added)) => Ok(PantsuTagInfo {
                        tag: PantsuTag { tag_type: PantsuTagType::deserialize(&tag_type)? , tag_name: tag_name },
                        tag_author: PantsuTagAuthor::deserialize(&tag_author)?,
                        date_added: NaiveDateTime::parse_from_str(&date_added, DATE_TIME_FORMAT)
                            .or_else(|e| Err(Error::InvalidDateFormat(e)))?
                    }),
                    Err(e) => Err(Error::SQLError(e))
                }
            })
            .collect::<Result<Vec<PantsuTagInfo>>>()?;
        Ok(rows)
    }

    pub fn repeat_vars(count: usize) -> String {
        assert_ne!(count, 0);
        let mut s = "?,".repeat(count);
        // Remove trailing comma
        s.pop();
        s
    }
}