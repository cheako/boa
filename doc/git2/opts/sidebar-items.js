initSidebarItems({"fn":[["strict_hash_verification","Controls whether or not libgit2 will verify that objects loaded have the expected hash. Enabled by default, but disabling this can significantly improve performance, at the cost of relying on repository integrity without checking it."],["strict_object_creation","Controls whether or not libgit2 will verify when writing an object that all objects it references are valid. Enabled by default, but disabling this can significantly improve performance, at the cost of potentially allowing the creation of objects that reference invalid objects (due to programming error or repository corruption)."]]});