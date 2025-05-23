use chin_tools::AResult;
use rusqlite::Connection;

pub fn has_table(conn: &Connection, table_name: &str) -> AResult<bool> {
    let mut stmt =
        conn.prepare("SELECT name FROM sqlite_master where type ='table' and name = ?")?;
    let name = stmt.query_row(&[table_name], |row| row.get::<_, String>(0))?;
    Ok(name == table_name)
}
