git-test-annotate is a tool that scans a git repository and produces
annotations related to tests.

Usage: ```git-test-annotate <path-to-repo>```

The tool prints to stdout two kinds of annotations:

1. File annotations, marking repository files in the current HEAD as Test,
   NotTest or Ignore. Files are marked as Ignore if they are not text or are
   translations. Files are considered Test if they are not Ignore and have
   the string "test" anywhere in their path inside the repository.

   ```[file] <path-to-file> <size-in-bytes> <Test/NotTest/Ignore>```

2. Commit annotations, marking commits as Test or NotTest, depending on whether
   they change files that have "test" anywher in the their path insider the
   repository.

   ```[commit] <commit-id> <Test/NotTest>```
