// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{env, fs};

fn main() {
    env::current_dir()
        .map(|root| {
            license::check_and_generate_license_headers(
                root,
                |path| fs::read_to_string(path).unwrap(),
                |path, content| {
                    fs::write(path, content.as_bytes());
                },
            )
        })
        .unwrap()
}

mod license {
    use chrono::Datelike;
    use lazy_static::lazy_static;
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};
    use walkdir::WalkDir;

    lazy_static! {
        static ref YEAR: String = chrono::Utc::now().year().to_string();
        static ref LICENCE_HEADER_PREFIX: String = "// Copyright".to_string();
        static ref LICENSE_HEADER: String = format!(
            r#"{} {} Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
"#,
            LICENCE_HEADER_PREFIX.to_string(),
            YEAR.to_string()
        );
        static ref LICENSED_EXTENSIONS: HashSet<String> = HashSet::from(["rs".to_string()]);
        static ref SKIP_DIR_NAMES: HashSet<String> = HashSet::from(["target".to_string()]);
    }

    pub(crate) fn check_and_generate_license_headers(
        root: PathBuf,
        read_file: fn(&Path) -> String,
        write_file: fn(&Path, String) -> (),
    ) {
        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Some(new_content) = overwrite_with_license_content(path, read_file) {
                write_file(path, new_content)
            }
        }
    }

    fn overwrite_with_license_content(
        path: &Path,
        read_file: fn(&Path) -> String,
    ) -> Option<String> {
        let extension = path.extension();
        if extension.is_none() {
            return None;
        }
        let extension_str = extension.unwrap().to_str().unwrap();
        if LICENSED_EXTENSIONS.contains(extension_str) {
            if let Some(content) = needs_license_header(path, read_file) {
                return Some(concat(LICENSE_HEADER.as_str(), content));
            }
        }
        None
    }

    /// Return Some(content) if file needs license appended.
    fn needs_license_header(path: &Path, read_file: fn(&Path) -> String) -> Option<String> {
        let content = read_file(path);
        if content.starts_with(&LICENSE_HEADER.to_string()) {
            None
        } else if content.starts_with(&LICENCE_HEADER_PREFIX.to_string()) {
            // assume file starts with outdated license comment,
            // find first line break that is uncommented
            let split = content.split_inclusive("\n");
            let mut index: usize = 0;
            for line in split {
                if line == "\n" {
                    return Some(content[index..].to_string());
                } else {
                    index = index + line.len();
                }
            }
            return None;
        } else {
            // no license comment, append licence at the start with the linebreak
            Some(concat("\n", content))
        }
    }

    fn concat(s1: &str, s2: String) -> String {
        let mut result = s1.to_string();
        result.push_str(s2.as_str());
        result
    }
}
