#[cfg(test)]
mod tests {
    use globset::{Glob, GlobSetBuilder};

    const PATTERNS: [&str; 14] = [
        // directory
        ".coverage",
        ".DS_Store",
        // ".egg-info",
        ".cache",
        ".mypy_cache",
        ".pylint_cache",
        ".pytest_cache",
        ".ruff_cache",
        "__pycache__",
        // file
        ".bash_history",
        ".log",
        ".o",
        ".pyc",
        ".python_history",
        "pip-log.txt",
    ];

    const NOT_MATCH_EXAMPLES: [&str; 3] = [".logending", ".ofine", ".offline"];

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_globset_single() {
        let glob = Glob::new("*.pyc").unwrap().compile_matcher();

        assert!(glob.is_match("demo.pyc"));
        assert!(glob.is_match("foo/bar.pyc"));
        assert!(glob.is_match("abc/def/ghi/jkl/ok.pyc"));
    }

    #[test]
    fn test_globset_multi() {
        let mut builder = GlobSetBuilder::new();
        for pattern in PATTERNS {
            builder.add(Glob::new(pattern).unwrap());
        }
        let set = builder.build().unwrap();
        for pattern in PATTERNS {
            assert!(set.is_match(pattern));
        }
    }

    #[test]
    fn test_globset_multi_not_match() {
        let mut builder = GlobSetBuilder::new();
        for pattern in PATTERNS {
            builder.add(Glob::new(pattern).unwrap());
        }
        let set = builder.build().unwrap();
        for pattern in NOT_MATCH_EXAMPLES {
            assert!(!set.is_match(pattern));
        }
    }
}
