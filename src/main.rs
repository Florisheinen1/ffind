use clap::{Arg, ArgAction, Command, ArgGroup, builder::TypedValueParser, Error, error::ContextKind};

use std::path::PathBuf;

#[derive(Clone)]
struct PathBufParser {}

impl PathBufParser {
	pub fn new() -> Self {Self {}}
}

impl TypedValueParser for PathBufParser {
	type Value = PathBuf;

	fn parse_ref(
		&self,
		cmd: &Command,
		arg: Option<&Arg>,
		value: &std::ffi::OsStr,
	) -> Result<Self::Value, Error>{
		// First, get the typed value as string
		let value: &str = match value.to_str() {
			Some(str_val) => str_val,
			None => return Err({
				let mut parse_error = Error::new(clap::error::ErrorKind::InvalidUtf8);
				parse_error.insert(ContextKind::InvalidArg, clap::error::ContextValue::String(arg.expect("Requires argument").to_string()));
				parse_error.insert(ContextKind::InvalidValue, clap::error::ContextValue::String(value.to_string_lossy().into()));		
				parse_error.format(&mut cmd.clone())
			})
		};

		// Then, convert to a PathBuf. Needs to be a directory		
		let path = PathBuf::from(value);
		if path.is_dir() {
			Ok(path)
		} else {
			Err({
				let mut path_error = Error::new(clap::error::ErrorKind::InvalidValue).with_cmd(cmd);
				path_error.insert(ContextKind::InvalidArg, clap::error::ContextValue::String(arg.expect("Argument required").to_string()));
				path_error.insert(ContextKind::InvalidValue, clap::error::ContextValue::String(value.to_string()));
				path_error.format(&mut cmd.clone())
			})
		}
	}
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
		.value_parser(PathBufParser::new())
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
			.multiple(true)
		)
		.get_matches();

	println!("Keyword: {:?}", cmd_matches.get_one::<String>("keyword").expect("Required"));
	println!("Should recurse: {:?}", cmd_matches.get_one::<bool>("recurse").expect("Required"));
	println!("Look for names: {:?}", cmd_matches.get_one::<bool>("names").expect("Required"));
	println!("Look for content: {:?}", cmd_matches.get_one::<bool>("contents").expect("Required"));
	println!("Look in folder: {:?}", cmd_matches.get_one::<PathBuf>("directory").expect("Required"));

	// let a = PathBuf::from(r"src/main.rs");
	// println!("Is file: {}", a.is_file());
	// println!("Defualt pathbuf: {}", PathBuf::from("src//* fef main.rs").display());
	// println!("Default pathbuf is folder: {}", PathBuf::from("./").is_dir());
}