//!
//! Copyright (c) 2019 Alan D. Salewski <salewski@att.net>
//!
//!     This program is free software; you can redistribute it and/or modify
//!     it under the terms of the GNU General Public License as published by
//!     the Free Software Foundation; either version 2 of the License, or
//!     (at your option) any later version.
//!
//!     This program is distributed in the hope that it will be useful,
//!     but WITHOUT ANY WARRANTY; without even the implied warranty of
//!     MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//!     GNU General Public License for more details.
//!
//!     You should have received a copy of the GNU General Public License
//!     along with this program; if not, write to the Free Software Foundation,
//!     Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301,, USA.
//!

//! ads-mutt-index-format-custom4 is a rewrite of the
//! `ads-mutt-index-format-custom` program in Rust ("rust-lang"), but is
//! derived from both that bash script and the earlier
//! `ads-mutt-index-format-custom3` golang program.

extern crate regex;

#[macro_use]
extern crate lazy_static;

use std::env;

use std::string::String;

use std::vec::Vec;

use regex::Regex;

const PROG: &str = "ads-mutt-index-format-custom4";

// static BE_VERBOSE: bool = false;
static BE_VERBOSE: bool = true;


// Our regex is compiled at most once per program execution, upon the first
// dereference of RE_EXPECTED_PATTERN (which, in future versions, will not
// ever happen when the user provides the '--help' or '--version' command line
// options).
//
lazy_static! {
    static ref RE_EXPECTED_PATTERN: Regex
        = Regex::new( r##"(?x)  # enable "significant whitespace" mode

            ^([[:space:]]*[[:digit:]]{1,}[[:space:]]{1,}[^\[]{1,})  # capture group 1
            ([\[]S:)                                                # capture group 2
            (                                                       # capture group 3 (start of)

            [[:digit:]]{4}\-[[:digit:]]{1,2}\-[[:digit:]]{1,2}[[:space:]]{1,}[[:digit:]]{1,2}:[[:digit:]]{1,2}
            )(
            :[[:digit:]]{1,2}
            []]
            [[:space:]]{1,}
            )(
            [[:digit:]]{4}\-[[:digit:]]{1,2}\-[[:digit:]]{1,2}[[:space:]]{1,}[[:digit:]]{1,2}:[[:digit:]]{1,2}
            )(
            :[[:digit:]]{1,2}[[:space:]]{1,}
            )
            [\[]LIST:
            ([[:space:]]*[^]]{1,})
            []]
            (
            [[:space:]]{1,}.*)
        "## ).unwrap();
}

fn main() {

    // FIXME: maybe use OsString, instead, to allow for data in busted encoding on input
    //
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!( "{} (error): required mutt pager_format line not provided; bailing out", PROG );
    }

    let orig_string = &args[1];

    let outp_string = String::new();

    // if ! RE_EXPECTED_PATTERN.is_match( orig_string ) {
    let captures = RE_EXPECTED_PATTERN.captures( orig_string );
    if captures.is_none() {
    // match captures {

    //     None => {

            eprintln!( "{} (warning): input line did not match regex; passing through unchanged", PROG );

            println!( "{}", orig_string );

            return;
        // },

        // Some( caps ) => {
        // }
    }

    if BE_VERBOSE {
        eprintln!( "{} (info): input line matched regex", PROG );
    }

    println!("Hello, world!");
}
