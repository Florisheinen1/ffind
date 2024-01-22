use clap::{Arg, ArgAction, Command, ArgGroup};

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
		.help("Search through content files");

	let cmd_matches = Command::new("MyApp")
		.version("1.0")
		.author("Floris Heinen")
		.about("Finding for Floris!")
		.arg(keyword)
		.arg(names)
		.arg(contents)
		.arg(recurse)
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
}