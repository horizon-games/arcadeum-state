/*
 * Copyright 2020 Horizon Blockchain Games Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! ABI versioning utilities

/// Generates a module exporting a canonical digest of the contents of the files and directories at the given paths.
///
/// The module is written to the given module path.
/// The version is exported as a constant with the given identifier.
///
/// To use the generated code, it must be [`include!`](https://doc.rust-lang.org/std/macro.include.html)d in your code:
///
/// ```ignore
/// include!(concat!(env!("OUT_DIR"), "/generated.rs"));
/// ```
///
/// The traversal of paths is deterministic.
/// Directories are traversed recursively.
///
/// # Examples
///
/// ```
/// let mut module = std::path::PathBuf::new();
/// module.push(std::env::var("OUT_DIR").unwrap());
/// module.push("generated.rs");
///
/// arcadeum::tag(
///     module,
///     "VERSION",
///     ["Cargo.toml", "Cargo.lock", "build.rs", "src"].iter(),
/// )
/// .unwrap();
/// ```
pub fn tag<P: AsRef<std::path::Path>>(
    module: impl AsRef<std::path::Path>,
    identifier: &str,
    paths: impl Iterator<Item = P>,
) -> std::io::Result<()> {
    let version = version(paths)?;

    std::fs::write(
        module,
        format!(
            "// {}\npub const {}: [u8; {}] = {:?};\n",
            crate::utils::hex(&version),
            identifier,
            version.len(),
            &version,
        ),
    )
}

/// Generates a canonical digest of the contents of the files and directories at the given paths.
///
/// The traversal of paths is deterministic.
/// Directories are traversed recursively.
///
/// # Examples
///
/// ```
/// println!(
///     "{}",
///     arcadeum::utils::hex(
///         &arcadeum::version::version(["Cargo.toml", "Cargo.lock", "build.rs", "src"].iter())
///             .unwrap()
///     )
/// );
/// ```
pub fn version<P: AsRef<std::path::Path>>(
    paths: impl Iterator<Item = P>,
) -> std::io::Result<Vec<u8>> {
    let mut keccak = tiny_keccak::Keccak::new_keccak256();

    scan(&mut keccak, paths)?;

    let mut version = [0; 32];
    keccak.finalize(&mut version);
    Ok(version.to_vec())
}

fn scan<P: AsRef<std::path::Path>>(
    keccak: &mut tiny_keccak::Keccak,
    paths: impl Iterator<Item = P>,
) -> std::io::Result<()> {
    let mut paths: Vec<_> = paths.map(|path| path.as_ref().to_owned()).collect();
    paths.sort_unstable();

    for path in &paths {
        let metadata = std::fs::metadata(path)?;

        if metadata.is_file() {
            keccak.update(&std::fs::read(path)?);
        } else if metadata.is_dir() {
            scan(
                keccak,
                std::fs::read_dir(path)?
                    .map(|entry| entry.unwrap().path())
                    .collect::<Vec<_>>()
                    .iter(),
            )?;
        }
    }

    Ok(())
}
