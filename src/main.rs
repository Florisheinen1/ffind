use clap::{
    builder::TypedValueParser, error::ContextKind, Arg, ArgAction, ArgGroup, Command, Error,
};

use std::path::PathBuf;

use std::fs;

#[derive(Clone)]
struct DirectoryParser {}
impl DirectoryParser {
    pub fn new() -> Self {
        Self {}
    }
}
impl TypedValueParser for DirectoryParser {
    type Value = Directory;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, Error> {
        // First, get the typed value as string
        let value: &str = match value.to_str() {
            Some(str_val) => str_val,
            None => {
                return Err({
                    let mut parse_error = Error::new(clap::error::ErrorKind::InvalidUtf8);
                    parse_error.insert(
                        ContextKind::InvalidArg,
                        clap::error::ContextValue::String(
                            arg.expect("Requires argument").to_string(),
                        ),
                    );
                    parse_error.insert(
                        ContextKind::InvalidValue,
                        clap::error::ContextValue::String(value.to_string_lossy().into()),
                    );
                    parse_error.format(&mut cmd.clone())
                })
            }
        };

        // Then, convert to a PathBuf. Needs to be a directory
        let path = PathBuf::from(value);
        return match Directory::from(path) {
            Ok(dir) => Ok(dir),
            Err(_) => {
                let mut path_error = Error::new(clap::error::ErrorKind::InvalidValue).with_cmd(cmd);
                path_error.insert(
                    ContextKind::InvalidArg,
                    clap::error::ContextValue::String(arg.expect("Argument required").to_string()),
                );
                path_error.insert(
                    ContextKind::InvalidValue,
                    clap::error::ContextValue::String(value.to_string()),
                );
                return Err(path_error.format(&mut cmd.clone()));
            }
        };
    }
}

#[derive(Clone, Debug)]
struct Directory {
    path: PathBuf,
}

impl Directory {
    fn from(path: PathBuf) -> Result<Directory, &'static str> {
        if path.is_dir() {
			let canonicalized = path.canonicalize().expect("Could not canonicalize path");
            Ok(Directory { path: canonicalized })
        } else {
            Err("Nope")
        }
    }
}

impl Walkable for Directory {
    fn walk(&self, recurse: bool, include_filenames: bool, include_file_contents: bool, keyword: &str) -> Vec<Occurrence> {
        let mut occurrences: Vec<Occurrence> = vec![];

        // First, check the name of the folder itself
        if include_filenames {
            let dir_name = self
                .path.as_path()
                .file_name()
                .expect("Failed to get folder name")
                .to_string_lossy();
            if dir_name.contains(keyword) {
                occurrences.push(Occurrence::FileName {
                    matching_text: keyword.to_string(),
                    path: self.path.clone(),
                });
            }
        }

        // Then, walk each child in this folder
		if let Ok(dir_entry) = self.path.read_dir() {
			for x in dir_entry {
				if let Ok(entry) = x {
					// If we recurse, also walk the children directories
					if recurse {
						if let Ok(dir) = Directory::from(entry.path()) {
							occurrences.extend(dir.walk(recurse, include_filenames, include_file_contents, keyword));
							continue;
						}
					}
	
					// Walk the children files
					if let Ok(file) = File::from(entry.path()) {
						occurrences.extend(file.walk(recurse, include_filenames, include_file_contents, keyword));
					}
				} else {
					println!("Failed to aquire item in directory");
				}
			}
		}
        return occurrences;
    }
}

trait Walkable {
    fn walk(&self, recurse: bool, include_filenames: bool, include_file_contents: bool, keyword: &str) -> Vec<Occurrence>;
}

struct File {
    path: PathBuf,
}
impl File {
    fn from(path: PathBuf) -> Result<File, &'static str> {
        if path.is_file() {
            Ok(File { path })
        } else {
            Err("Failed to create File object from path")
        }
    }
}

impl Walkable for File {
    fn walk(&self, _: bool, include_filenames: bool, include_file_contents: bool, keyword: &str) -> Vec<Occurrence> {
        let mut occurrences: Vec<Occurrence> = vec![];

        if include_filenames {
            let filename: String = self.path.file_name().expect("Failed to get filename").to_string_lossy().to_string();
            if filename.contains(keyword) {
                occurrences.push(Occurrence::FileName {
                    matching_text: keyword.to_string(),
                    path: self.path.clone()
                })
            }
        }
		
		if include_file_contents {
			occurrences.extend(get_occurrences_in_file_contents(self, keyword));
		}

        return occurrences;
    }
}

#[derive(Debug)]
enum Occurrence {
    FileName {
        matching_text: String,
        path: PathBuf,
    },
    FileContent {
        matching_text: String,
        path: PathBuf,
        line_number: usize
    }
}

// floris

/// Lists occurrences of keyword in contents of file
fn get_occurrences_in_file_contents(file: &File, keyword: &str) -> Vec<Occurrence> {
    let mut occurrences: Vec<Occurrence> = vec![];

    let contents = if let Ok(c) = fs::read_to_string(file.path.clone()) {
        c
    } else {
        // println!("Skipping search in file: {:?}", file.path); // TODO: Resolve this
        return vec![];
    };

    let mut start_it = contents.chars();
    let mut line_counter: usize = 0;

    let mut keyword_it_start = keyword.chars();
    keyword_it_start.next();

    'start_loop: while let Some(start_char) = start_it.next() {
        if start_char == '\n' {
            line_counter += 1;
        }

        if !keyword.starts_with(start_char) {
            continue 'start_loop;
        }

        let mut follow_it = start_it.clone();
        let mut keyword_it = keyword_it_start.clone();

        // Go through each keyword char and check if it matches the following content char
        while let Some(keyword_char) = keyword_it.next() {
            if let Some(follow_char) = follow_it.next() {
                if keyword_char != follow_char {
                    // This means the found characters do not match the keyword
                    continue 'start_loop;
                }
            } else {
                // If there are no other content chars, we cannot match the remainder of the keyword
                continue 'start_loop;
            }
        }

        // At this point, we know the keyword has been found
        occurrences.push(Occurrence::FileContent {
            matching_text: keyword.to_string(),
            path: file.path.clone(),
            line_number: line_counter,
        });
    }

    return occurrences;
}

fn main() {
    let keyword = Arg::new("keyword")
        .required(true)
        .action(ArgAction::Set)
        .help("The keyword that needs to be searched for");

    let recurse = Arg::new("recurse")
        .long("recurse")
        .short('r')
        .action(ArgAction::SetTrue)
        .required(false)
        .help("Search recursively through folders");

    let names = Arg::new("names")
        .long("name")
        .short('n')
        .action(ArgAction::SetTrue)
        .help("Search through name of files");

    let contents = Arg::new("contents")
        .long("content")
        .short('c')
        .action(ArgAction::SetTrue)
        .help("Search through content of files");

    let location = Arg::new("directory")
        .long("dir")
        .short('d')
        .action(ArgAction::Set)
        .value_parser(DirectoryParser::new())
        .default_value("./")
        .help("Search in given folder");

    let cmd_matches = Command::new("MyApp")
        .version("1.0")
        .author("Floris Heinen")
        .about("Finding for Floris!")
        .arg(keyword)
        .arg(names)
        .arg(contents)
        .arg(recurse)
        .arg(location)
        .group(
            ArgGroup::new("SearchFlags")
                .args(["names", "contents"])
                .required(true)
                .multiple(true),
        )
        .get_matches();

    let keyword = cmd_matches.get_one::<String>("keyword").expect("Required");
    let should_recurse = cmd_matches.get_one::<bool>("recurse").expect("Required");
    let include_filenames = cmd_matches.get_one::<bool>("names").expect("Required");
    let include_file_contents = cmd_matches.get_one::<bool>("contents").expect("Required");

    let search_directory = cmd_matches
        .get_one::<Directory>("directory")
        .expect("Required");

	println!("Search dir: {:?}", search_directory);

    let occurrences = search_directory.walk(*should_recurse, *include_filenames, *include_file_contents, &keyword);

    for occurrence in occurrences {
		match occurrence {
			Occurrence::FileName { matching_text, path } => println!("'{}' found in filename: '{:?}'", matching_text, path),
			Occurrence::FileContent { matching_text, path, line_number } => println!("'{}' found on line {} in file: '{:?}'", matching_text, line_number, path)
		}
    }

}

// flo
