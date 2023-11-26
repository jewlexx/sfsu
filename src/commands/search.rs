use std::{
    ffi::OsStr,
    fs::{read_dir, DirEntry},
    io::Error,
};

use colored::Colorize;
use itertools::Itertools;
use rayon::prelude::*;

use clap::{Parser, ValueEnum};
use regex::Regex;

use sfsu::{
    buckets,
    output::sectioned::{Children, Section, Sections, Text},
    packages::manifest::StringOrArrayOfStringsOrAnArrayOfArrayOfStrings,
};

use sfsu::packages::{is_installed, CreateManifest, Manifest};
use strum::Display;

#[derive(Debug, Default, Copy, Clone, ValueEnum, Display, Parser)]
#[strum(serialize_all = "snake_case")]
enum SearchMode {
    #[default]
    Name,
    Binary,
    Both,
}

impl SearchMode {
    pub fn match_names(self) -> bool {
        matches!(self, SearchMode::Name | SearchMode::Both)
    }

    pub fn match_binaries(self) -> bool {
        matches!(self, SearchMode::Binary | SearchMode::Both)
    }
}

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The regex pattern to search for, using Rust Regex syntax")]
    pattern: String,

    #[clap(
        short,
        long,
        help = "Whether or not the pattern should match case-sensitively"
    )]
    case_sensitive: bool,

    #[clap(short, long, help = "The bucket to exclusively search in")]
    bucket: Option<String>,

    #[clap(short, long, help = "Only search installed packages")]
    installed: bool,

    #[clap(short, long, help = "Search mode to use", default_value_t)]
    mode: SearchMode,
}

enum MatchOutput {
    PackageName,
    BinaryName(String),
}

impl std::fmt::Display for MatchOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchOutput::PackageName => Ok(()),
            MatchOutput::BinaryName(name) => write!(f, "{}", name.bold()),
        }
    }
}

struct MatchCriteria(Vec<MatchOutput>);

impl MatchCriteria {
    pub fn has_bin(&self) -> bool {
        self.0
            .iter()
            .any(|m| matches!(m, MatchOutput::BinaryName(_)))
    }
}

fn match_criteria(
    file_name: &str,
    manifest: &Manifest,
    mode: SearchMode,
    pattern: &Regex,
) -> MatchCriteria {
    // use std::rc::Rc;
    // let name = Rc::new(file_name);

    let file_name = file_name.to_string();

    let mut output = vec![];

    if mode.match_names() && pattern.is_match(&file_name) {
        output.push(MatchOutput::PackageName);
    }
    if mode.match_binaries() {
        let binaries = manifest
            .bin
            .clone()
            .map(StringOrArrayOfStringsOrAnArrayOfArrayOfStrings::to_vec)
            .unwrap_or_default();

        let binary_names = binaries
            .into_iter()
            .filter(|binary| pattern.is_match(binary))
            .filter_map(|b| {
                if pattern.is_match(&b) {
                    Some(MatchOutput::BinaryName(b.clone()))
                } else {
                    None
                }
            });

        output.extend(binary_names);
    }

    MatchCriteria(output)
}

fn parse_output(
    file: &DirEntry,
    bucket: impl AsRef<str>,
    installed_only: bool,
    pattern: &Regex,
    mode: SearchMode,
) -> Option<Section<Text<String>>> {
    let path = file.path();

    if !matches!(path.extension().and_then(OsStr::to_str), Some("json")) {
        return None;
    }

    // This may be a bit of a hack, but it works
    let file_name = path
        .with_extension("")
        .file_name()
        .map(|osstr| osstr.to_string_lossy().to_string());
    let package_name = file_name.unwrap();

    // TODO: Better display of output
    match Manifest::from_path(file.path()) {
        Ok(manifest) => {
            let match_output = match_criteria(&package_name, &manifest, mode, pattern);

            if match_output.0.is_empty() {
                return None;
            }

            // TODO: Refactor to remove pointless binary matching on name-only search mode
            // TODO: Fix error parsing manifests

            let is_installed = is_installed(&package_name, Some(bucket));
            if installed_only && !is_installed {
                return None;
            }

            let styled_package_name = if package_name == pattern.to_string() {
                package_name.bold().to_string()
            } else {
                package_name
            };

            let installed_text = if is_installed && !installed_only {
                "[installed] "
            } else {
                ""
            };

            let title = format!(
                "{styled_package_name} ({}) {installed_text}",
                manifest.version
            );

            debug!("Matched {title}");

            let package = if match_output.has_bin() {
                let bins = match_output
                    .0
                    .iter()
                    .filter_map(|output| match output {
                        MatchOutput::BinaryName(bin) => Some(Text::new(format!(
                            "{}{}\n",
                            sfsu::output::sectioned::WHITESPACE,
                            bin.bold()
                        ))),
                        MatchOutput::PackageName => None,
                    })
                    .collect_vec();

                Section::new(Children::Multiple(bins))
            } else {
                Section::new(Children::None)
            }
            .with_title(title);

            Some(package)
        }
        Err(_) => None,
    }
}

impl super::Command for Args {
    fn run(self) -> Result<(), anyhow::Error> {
        let (bucket, raw_pattern) = if self.pattern.contains('/') {
            let mut split = self.pattern.splitn(2, '/');

            // Bucket flag overrides bucket/package syntax
            let bucket = self.bucket.unwrap_or(split.next().unwrap().to_string());
            let pattern = split.next().unwrap();

            (Some(bucket), pattern.to_string())
        } else {
            (self.bucket, self.pattern)
        };

        let pattern = {
            Regex::new(&format!(
                "{}{}",
                if self.case_sensitive { "" } else { "(?i)" },
                &raw_pattern
            ))
            .expect("Invalid Regex provided. See https://docs.rs/regex/latest/regex/ for more info")
        };

        debug!("Matching with pattern {pattern}");

        let all_scoop_buckets = buckets::Bucket::list_all()?;

        let scoop_buckets = {
            if let Some(bucket) = bucket {
                all_scoop_buckets
                    .into_iter()
                    .filter(|scoop_bucket| {
                        let path = scoop_bucket.path();
                        match path.components().last() {
                            Some(x) => x.as_os_str() == bucket.as_str(),
                            None => false,
                        }
                    })
                    .collect()
            } else {
                all_scoop_buckets
            }
        };

        let mut matches = scoop_buckets
            .par_iter()
            .filter_map(|bucket| {
                // Ignore loose files in the buckets dir
                if !bucket.path().is_dir() {
                    return None;
                }

                let bucket_path = {
                    let bk_path = bucket.path().join("bucket");

                    if bk_path.exists() {
                        bk_path
                    } else {
                        bucket.path().to_owned()
                    }
                };

                let bucket_contents = read_dir(bucket_path)
                    .and_then(Iterator::collect::<Result<Vec<_>, _>>)
                    .unwrap();

                let matches = bucket_contents
                    .par_iter()
                    .filter_map(|file| {
                        parse_output(file, bucket.name(), self.installed, &pattern, self.mode)
                    })
                    .collect::<Vec<_>>();

                if matches.is_empty() {
                    None
                } else {
                    Some(Ok::<_, Error>(
                        Section::new(Children::Multiple(matches))
                            // TODO: Remove quotes and bold bucket name
                            .with_title(format!("'{}' bucket:", bucket.name())),
                    ))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        matches.par_sort_by_key(|x| x.title.clone());

        print!("{}", Sections::from_vec(matches));

        Ok(())
    }
}
