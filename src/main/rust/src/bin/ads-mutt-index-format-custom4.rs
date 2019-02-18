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
extern crate unicode_segmentation;

#[macro_use]
extern crate lazy_static;

use std::env;

use std::string::String;

use std::vec::Vec;

use regex::Captures;
use regex::Regex;

// See:
//     https://github.com/unicode-rs/unicode-segmentation
//     https://unicode-rs.github.io/unicode-segmentation/unicode_segmentation/index.html
//     https://crates.io/crates/unicode-segmentation
//
use unicode_segmentation::UnicodeSegmentation;

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

    let captures = RE_EXPECTED_PATTERN.captures( orig_string );
    if captures.is_none() {

        eprintln!( "{} (warning): input line did not match regex; passing through unchanged", PROG );

        println!( "{}", orig_string );

        return;
    }
    // captures is known to be Some(v) here (that is, it is known to not be None)

    if BE_VERBOSE {
        eprintln!( "{} (info): input line matched regex", PROG );
    }

    let caps: Captures = captures.unwrap();

    let whatev1 = &caps[1];
    let whatev2 = &caps[2];     // "[S:"

    let dt_lft  = &caps[3];

    let whatev3 = &caps[4];     // seconds portion of left-hand date, closing bracket, plus spaces

    let dt_rit  = &caps[5];

    let whatev4 = &caps[6];     // seconds portion of right-hand date, closing bracket, plus spaces

    let listnm_raw = &caps[7];  // list name (or mail file name, such as 'ads'), possibly surrounded by whitespace

    let whatev5 = &caps[8];     // the rest of the index format line

    let mut outp_string = String::with_capacity( orig_string.len() );

    if dt_lft == dt_rit {

        let mut str_with_olength = String::with_capacity( whatev2.len() + dt_lft.len() + whatev3.len() );
        str_with_olength.push_str( whatev2 );
        str_with_olength.push_str( dt_lft  );
        str_with_olength.push_str( whatev3 );  // full string segment we'll be replacing

        // CAREFUL: We need to count the graphemes in str_with_olength, not
        //          the number of bytes (would only happen to give the correct
        //          count if the text content happened to be all 7-bit ASCII)
        //          or even the number of chars (would be incorrect when
        //          combining chars are present, or bytes from scripts that
        //          are not considered combining by Unicode but which
        //          nevertheless rely on character composition).
        //
        //          See also:
        //
        //              • "Dark corners of Unicode" by Eevee, especially the section "Combining characters and character width":
        //                https://eev.ee/blog/2015/09/12/dark-corners-of-unicode/
        //
        //              • "Let’s Stop Ascribing Meaning to Code Points" by Manish Goregaokar
        //                https://manishearth.github.io/blog/2017/01/14/stop-ascribing-meaning-to-unicode-code-points/
        //
        let count_of_needed_spaces = UnicodeSegmentation::graphemes( str_with_olength.as_str(),
                                                                     true /* extended grapheme clusters? (as opposed to "legacy grapheme clusters") */ )
                                    .count();

        // outp_string.push_str( format!( "%s%*s%s%s`, whatev1, count_of_needed_spaces, ``, dt_rit, whatev4 )
        // outp_string.push_str( &format!( "{}{:2$}{}{}", whatev1, "", count_of_needed_spaces, dt_rit, whatev4 ) );
        // outp_string.push_str( format!( "{}", whatev1 ).as_str() );
        // outp_string.push_str( &format!( "{}", whatev1 ) );
        outp_string.push_str( &format!( "{}{}", whatev1, "" ) );

// // // DEBUG go
// //         fmt.Printf( "O:%s\n", orig_string )
// //         fmt.Printf( "N:%s\n", outp_string  )
// // // DEBUG end
//     } else {

//         // The dates are different, so "pass through" date values unchanged

//         if _BE_VERBOSE {
//             fmt.Fprintf( os.Stderr, "%s (info): dates are different; passing through unchanged\n", _PROG )
//         }

//         // Reconstruct original input string through the right-hand date portion.
//         outp_string += fmt.Sprintf( `%s%s%s%s%s%s`, whatev1, whatev2, dt_lft, whatev3, dt_rit, whatev4 )

//         // fmt.Println( orig_string )
    }


    println!("Hello, world!");
}
