#[allow(dead_code)]
pub static FIXTURES: &[&str] = &[
  "/foo/../../../bar",
  "a//b//../b",
  "/foo/../../../bar",
  "a//b//./c",
  "a//b//.",
  "/a/b/c/../../../x/y/z",
  "///..//./foo/.//bar",
  "bar/foo../../",
  "bar/foo../..",
  "bar/foo../../baz",
  "bar/foo../",
  "bar/foo..",
  "../foo../../../bar",
  "../foo../../../bar",
  "../.../.././.../../../bar",
  "../.../.././.../../../bar",
  "../../../foo/../../../bar",
  "../../../foo/../../../bar/../../",
  "../foobar/barfoo/foo/../../../bar/../../",
  "../.../../foobar/../../../bar/../../baz",
  "foo/bar\\baz",
  "/a/b/c/../../../",
  "a/b/c/../../../",
  "a/b/c/../../..",
  "",
  // Deep paths (8 components - at SmallVec boundary)
  "a/b/c/d/e/f/g/h",
  "/level1/level2/level3/level4/level5/level6/level7/level8",
  // Deep paths (9-12 components - just over SmallVec inline capacity)
  "a/b/c/d/e/f/g/h/i",
  "/level1/level2/level3/level4/level5/level6/level7/level8/level9",
  "comp1/comp2/comp3/comp4/comp5/comp6/comp7/comp8/comp9/comp10",
  "/root/sub1/sub2/sub3/sub4/sub5/sub6/sub7/sub8/sub9/sub10/file.txt",
  // Deep paths (15+ components)
  "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o",
  "/level1/level2/level3/level4/level5/level6/level7/level8/level9/level10/level11/level12/level13/level14/level15",
  // Very deep paths (20+ components)
  "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t",
  "/home/user/projects/company/backend/services/api/controllers/v2/handlers/auth/login/validate/token/refresh/generate/new",
  // Deep paths with dots for normalization
  "a/./b/./c/./d/./e/./f/./g/./h/./i/./j",
  "/level1/./level2/./level3/./level4/./level5/./level6/./level7/./level8/./level9",
  "a/b/../c/d/../e/f/../g/h/../i/j/../k/l/../m/n/../o/p",
  "/level1/level2/../level3/level4/../level5/level6/../level7/level8/../level9/level10",
  // Complex normalization with deep nesting
  "a/./b/../c/./d/../e/./f/../g/./h/../i/./j/../k/./l/../m/./n/../o/./p",
  "../../../a/b/c/../../d/e/f/../../g/h/i/../../j/k/l/../../m/n/o",
  "./a/b/./c/../d/./e/f/../../g/h/./i/../j/./k/l/../../m/./n/o/../p/q/./r",
];

#[allow(dead_code)]
pub static ABSOLUTE_PATHS: &[&str] = &[
  "/hello",
  "/usr/local/bin",
  "/home/user/documents/file.txt",
  "/var/log/system.log",
  "/etc/config/settings.json",
  "/tmp/cache/data",
  "/opt/application/lib",
  "/root/.ssh/id_rsa",
  "/mnt/storage/backup",
  "/dev/null",
  "/proc/cpuinfo",
  "/sys/class/net",
  "/boot/grub/grub.cfg",
  "/lib64/libc.so.6",
  "/usr/share/doc/readme.md",
  "/var/www/html/index.html",
  "/home/admin/.bashrc",
  "/etc/passwd",
  "/usr/bin/python3",
  "/var/spool/mail",
  "/opt/tools/scripts/deploy.sh",
  // Deep absolute paths
  "/level1/level2/level3/level4/level5/level6/level7/level8/level9/level10/level11/level12",
  "/usr/local/share/doc/packages/example/tutorials/advanced/chapter1/section2/subsection3/page4.html",
  "/home/user/workspace/projects/company/backend/microservices/auth-service/src/controllers/v2/handlers/login.js",
  "/var/log/applications/production/cluster-01/node-03/services/api-gateway/2024/01/15/access.log",
];

/// Clean relative paths (no `.`/`..`/`//`/trailing `/`).
/// Used to benchmark absolutize on relative inputs (always allocates).
#[allow(dead_code)]
pub static RELATIVE_CLEAN: &[&str] = &[
  "foo",
  "foo/bar",
  "foo/bar/baz",
  "src/main.rs",
  "src/lib/utils/helpers.rs",
  "node_modules/@scope/package/dist/index.js",
  "a/b/c/d/e/f/g/h",
  "a/b/c/d/e/f/g/h/i/j",
  "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o",
];

/// Absolute paths that need normalization (contain `.`/`..`/`//`/trailing `/`).
/// Used to benchmark absolutize on dirty absolute inputs.
#[allow(dead_code)]
pub static DIRTY_ABSOLUTE: &[&str] = &[
  "/foo/../../../bar",
  "/a/b/c/../../../x/y/z",
  "///..//./foo/.//bar",
  "/a/b/c/../../../",
  "/foo/bar//baz/asdf/quux/..",
  "/usr/local/./bin/../lib/./share",
];

/// Paths that are already in normalized form on Unix.
/// No `.` or `..` components, no `//` doubled separators, no trailing `/`.
/// Used to benchmark the zero-allocation fast path.
#[allow(dead_code)]
pub static ALREADY_NORMALIZED_UNIX: &[&str] = &[
  // Single-component paths
  "foo",
  "file.txt",
  "/single",
  // Short relative paths (2-3 components)
  "foo/bar",
  "foo/bar/baz",
  "src/main.rs",
  // Short absolute paths
  "/",
  "/foo",
  "/foo/bar",
  "/usr/local/bin",
  // Medium paths (4-7 components) with dots in filenames
  "src/lib/utils/helpers.rs",
  "/home/user/.config/settings.json",
  "/var/log/app.2024.01.15.log",
  "node_modules/@scope/package/dist/index.js",
  "src/.hidden/file.rs",
  // Tricky: dots in component names that are NOT . or ..
  "..foo/bar",
  "foo..bar/baz",
  ".../.../foo",
  // At SmallVec boundary (8 components)
  "a/b/c/d/e/f/g/h",
  "/level1/level2/level3/level4/level5/level6/level7/level8",
  // Over SmallVec boundary (10-12 components)
  "a/b/c/d/e/f/g/h/i/j",
  "/usr/local/share/doc/packages/example/tutorials/advanced/chapter1/section2",
  "/var/log/applications/production/cluster-01/node-03/services/api-gateway/access.log",
  // Very deep paths (15-20 components)
  "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o",
  "/home/user/projects/company/backend/services/api/controllers/v2/handlers/auth/login/validate/token/refresh/generate/key/store/cache/data",
  // --- Additional workload ---
  // Single-component paths
  "bar",
  "image.png",
  "/root",
  // Short relative paths (2-3 components)
  "baz/qux",
  "baz/qux/corge",
  "tests/unit.rs",
  // Short absolute paths
  "/tmp",
  "/bar/baz",
  "/opt/bin/tool",
  // Medium paths with dots in filenames
  "lib/core/math/vector.rs",
  "/opt/app/.env.production",
  "/tmp/data/report.2024.q1.csv",
  "packages/@company/sdk/lib/index.mjs",
  "config/.secrets/keys.pem",
  // Tricky dot names
  "..bar/baz",
  "baz..qux/corge",
  ".../..../bar",
  // At SmallVec boundary (8 components)
  "p/q/r/s/t/u/v/w",
  "/alpha/bravo/charlie/delta/echo/foxtrot/golf/hotel",
  // Over SmallVec boundary (10-12 components)
  "p/q/r/s/t/u/v/w/x/y",
  "/opt/data/warehouse/etl/pipelines/transforms/staging/output/validated/reports",
  "/srv/apps/production/cluster-02/node-05/services/graphql-gateway/access.log",
  // Very deep paths (15-20 components)
  "p/q/r/s/t/u/v/w/x/y/z/aa/bb/cc/dd",
  "/srv/data/projects/org/team/repo/packages/core/src/modules/auth/handlers/v3/internal/process/queue/worker/task",
];

/// Paths that are already in normalized form on Windows.
#[allow(dead_code)]
pub static ALREADY_NORMALIZED_WINDOWS: &[&str] = &[
  "C:\\",
  "C:\\foo",
  "C:\\foo\\bar",
  "C:\\Users\\Admin\\Documents\\file.txt",
  "D:\\Projects\\rust\\src\\main.rs",
  "\\\\server\\share\\",
  "\\\\server\\share\\folder\\document.doc",
  "C:\\Windows\\System32\\drivers\\etc\\hosts",
  "C:\\Program Files\\Application\\bin\\app.exe",
  // At SmallVec boundary
  "C:\\level1\\level2\\level3\\level4\\level5\\level6\\level7\\level8",
  // Deep paths
  "C:\\a\\b\\c\\d\\e\\f\\g\\h\\i\\j\\k\\l\\m\\n\\o\\p\\q\\r\\s\\t",
  "D:\\home\\user\\workspace\\projects\\company\\backend\\microservices\\auth-service\\src\\controllers\\v2\\handlers",
  // --- Additional workload ---
  "E:\\",
  "E:\\bar",
  "E:\\bar\\baz",
  "C:\\Users\\Deploy\\.profile\\config.toml",
  "F:\\Games\\Steam\\steamapps\\common\\app.exe",
  "\\\\nas\\backup\\",
  "\\\\nas\\backup\\archives\\2024\\data.zip",
  "C:\\ProgramData\\Docker\\volumes\\db\\data",
  "D:\\workspace\\monorepo\\packages\\core\\dist\\index.cjs",
  // At SmallVec boundary
  "E:\\alpha\\bravo\\charlie\\delta\\echo\\foxtrot\\golf\\hotel",
  // Deep paths
  "E:\\p\\q\\r\\s\\t\\u\\v\\w\\x\\y\\z\\aa\\bb\\cc\\dd\\ee\\ff\\gg\\hh\\ii",
  "F:\\srv\\data\\projects\\org\\team\\repo\\packages\\core\\src\\modules\\auth\\handlers",
];
