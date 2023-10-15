
#[cfg(test)]
mod tests {
    use toml::Table;


    #[test]
    fn test_toml_basic() {
        let value = "foo = 'bar'".parse::<Table>().unwrap();
        assert_eq!(value["foo"].as_str(), Some("bar"));
    }
    #[test]
    fn test_toml_array() {
        let mut xs: Vec<String> = vec![];
        let value = "foo = ['bar', 'bat']".parse::<Table>().unwrap();
        let array = value["foo"].as_array().unwrap();
        for elem in array {
            xs.push(elem.to_owned().to_string());
        }
        assert!(xs.iter().any(|e| e.contains("bar")));
        assert!(xs.iter().any(|e| e.contains("bat")));
    }
    #[test]
    fn test_toml_table() {
        let table = "\
# rclean configuration
path = '.'
patterns = [
    # dirs
    '*/.coverage',
    '*/.DS_Store',
    '*/.cache',
    '*/.mypy_cache',
    '*/.pylint_cache',
    '*/.pytest_cache',
    '*/.ruff_cache',
    '*/__pycache__',
    # files
    '*/.bash_history',
    '.log',
    '.o',
    '.py[co]',
    '*/.python_history',
    '*/pip-log.txt',
]
dry_run = false
skip_confirmation = false
".parse::<Table>().unwrap();
        assert_eq!(table["path"].as_str(), Some("."));
        //let array = table["patterns"].as_array();
        assert_eq!(table["dry_run"].as_bool(), Some(false));
    }
}