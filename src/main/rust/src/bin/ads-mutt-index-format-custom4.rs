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

use std::env;

const PROG: &str = "ads-mutt-index-format-custom4";

fn main() {

    // FIXME: maybe use OsString, instead, to allow for data in busted encoding on input
    //
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!( "{} (error): required mutt pager_format line not provided; bailing out\n", PROG );
    }

    let orig_string = &args[1];

    let outp_string = String::new();

    println!("Hello, world!");
}
