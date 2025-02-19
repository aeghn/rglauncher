use std::fs::DirEntry;

use chin_tools::AResult;

pub fn walk_dir<F>(dirpath: &str, filter: Option<F>) -> AResult<Vec<DirEntry>>
where
    F: Fn(&str) -> bool,
{
    let mut queue = vec![];
    queue.push(dirpath.to_string());

    let mut result = vec![];

    while let Some(p) = queue.pop() {
        for rde in std::fs::read_dir(p)? {
            let de = rde?;
            let dpath = de.path();
            let path_str = dpath.to_str().unwrap().to_string();
            if let Some(f) = filter.as_ref() {
                if f(path_str.as_str()) {
                    result.push(de);
                }
            } else {
                result.push(de)
            }
            if dpath.is_dir() {
                queue.push(path_str);
            }
        }
    }

    Ok(result)
}
