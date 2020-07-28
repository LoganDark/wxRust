use std::path::PathBuf;
use std::process::Command;

use log::{info, LevelFilter};

fn main() {
	env_logger::builder()
		.filter_level(LevelFilter::Info)
		.filter_module("bindgen", LevelFilter::Error)
		.init();

	println!("cargo:rerun-if-env-changed=WXCONFIG");
	let wxconfig_path = std::env::var("WXCONFIG")
		.unwrap_or(String::from("wx-config"));

	Command::new(wxconfig_path.clone())
		.status()
		.expect("Ensure you have `wx-config` in your PATH, or specify the WXCONFIG environment variable!");

	// Beyond just the include paths, the flags returned by `wx-config` include
	// important target definitions that tell wxWidgets what platform it's on,
	// which means we really need those flags!
	//
	// Here's what they are on my system as an example:
	//   -I/usr/local/lib/wx/include/osx_cocoa-unicode-3.1
	//   -I/usr/local/include/wx-3.1 -DWXUSINGDLL -D__WXOSX_COCOA__ -D__WXMAC__
	//   -D__WXOSX__ -pthread
	//
	// Other than the include paths, that includes `WXUSINGDLL`,
	// `__WXOSX_COCOA__`, `__WXMAC__`, and `__WXOSX__`! Also the include paths
	// are kind of important to be able to detect dynamically!
	let cxx_flags = Command::new(wxconfig_path.clone())
		.arg("--cxxflags")
		.output()
		.expect("Couldn't get flags from `wx-config`");

	let cxx_flags = String::from_utf8(cxx_flags.stdout)
		.expect("`wx-config --cxxflags` outputted invalid UTF-8");

	info!("`wx-config` CXX flags: {}", cxx_flags);

	let cxx_flags = cxx_flags + " -std=c++14";
	let cxx_flags = shlex::split(cxx_flags.as_str())
		.expect("Couldn't split CXX flags!");

	info!("Final CXX flags: {:?}", cxx_flags);

	// `wx-config` also returns library paths, which are important to link to
	// the program after it's been compiled. Or else the bindings wouldn't work
	// and everyone would be unhappy.
	//
	// On my system they're:
	//   -L/usr/local/lib -pthread -lwx_osx_cocoau_xrc-3.1
	//   -lwx_osx_cocoau_webview-3.1 -lwx_osx_cocoau_stc-3.1
	//   -lwx_osx_cocoau_richtext-3.1 -lwx_osx_cocoau_ribbon-3.1
	//   -lwx_osx_cocoau_propgrid-3.1 -lwx_osx_cocoau_aui-3.1
	//   -lwx_osx_cocoau_gl-3.1 -lwx_osx_cocoau_media-3.1
	//   -lwx_osx_cocoau_html-3.1 -lwx_osx_cocoau_qa-3.1
	//   -lwx_osx_cocoau_core-3.1 -lwx_baseu_xml-3.1 -lwx_baseu_net-3.1
	//   -lwx_baseu-3.1
	//
	// Without these libs the program would crash at runtime, which is not what
	// you want when writing Rust. Especially not what you want when you include
	// a library that's supposed to handle the FFI for you!
	let libs = Command::new(wxconfig_path)
		.args(vec!["--libs", "all"])
		.output()
		.expect("Couldn't get libs from `wx-config`");

	let libs = String::from_utf8(libs.stdout)
		.expect("`wx-config --libs all` outputted invalid UTF-8");

	println!("cargo:rustc-flags={}", libs);

	info!("Generating bindings now");

	println!("cargo:rerun-if-changed=wrapper.hpp");
	let bindings = bindgen::Builder::default()
		.header("wrapper.hpp")
		.detect_include_paths(false)
		.clang_args(cxx_flags)
		.parse_callbacks(Box::new(bindgen::CargoCallbacks))
		.whitelist_type("wx.*")
		.opaque_type("(::)?std::.*")
		.generate()
		.expect("Couldn't make bindings");

	let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
	bindings.write_to_file(out_path.join("bindings.rs"))
		.expect("Couldn't write bindings");
}