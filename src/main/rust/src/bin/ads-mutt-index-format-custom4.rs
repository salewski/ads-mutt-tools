//!
//! Copyright (c) 2019, 2021 Alan D. Salewski <ads@salewski.email>
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

static BE_VERBOSE: bool = false;
// static BE_VERBOSE: bool = true;


// Our regex is compiled at most once per program execution, upon the first
// dereference of RE_EXPECTED_PATTERN (which, in future versions, will not
// ever happen when the user provides the '--help' or '--version' command line
// options).
//
lazy_static! {

    // UPDATE (2021-01-08): This is our new default folder-hook index_format:
    //
    //     \"%5C  %Z  %?M?%-9M&%4e/%-4E?  [S:%d]  %D  [LIST: %-16.16B]  %-30.30F (b: %6c; l: %6l) %?X?%2X&  ?  %s\""
    //
    // Note that in our regex, repeat operators in the form {1,N} are
    // sometimes provided where logically we mean {1,}. The 'N' value in such
    // cases is arbitrary, but significantly larger than any value we would
    // expect to see in the real world. The idea is that we provide bounds to
    // the regex. All such usages are commented.
    //
    static ref RE_EXPECTED_PATTERN: Regex
        = Regex::new( r##"(?x)  # enable "significant whitespace" mode

            ^
            (  # start capture group 1

               # '%5C' -- Leading space here is only present if the message
               #          number is less than 10,000.
               [[:space:]]*[[:digit:]]{1,15}    # The '15' here is just to keep it bounded

               # Our fields are separated by exactly two spaces
               [[:space:]]{2}

               # '%Z' -- Always exactly three characters wide -- one or more
               #         of which may be spaces
               .{3}

               # Our standard 2-char field separator
               [[:space:]]{2}
            )


            (  # start capture group 2

               # %M OR %e/%E -- Field is always at least 9 chars wide -- more
               #                if the data does not fit into that many char
               #                cells. Whitespace can be anyplace at the
               #                beginning or end of the value.
               (  # capture group 3
                  [[:space:]]*
               )
               (  # capture group 4
                  [^[:space:]]{1,15}    # The '15' here is just to keep it bounded
               )
               (  # capture group 5

                  # Regardless of how many non-whitespace chars were provided,
                  # the value will always be followed by at least two whitespace
                  # chars (our standard field separator), but maybe more than
                  # that (if there were fewer than 9 non-whitespace chars in
                  # the field value).
                  #
                  # '10' is the max (8 + 2). The max whitespace chars from the
                  # field will be 8 (if the entire field value was "1" and was left
                  # justified), plus the two for our standard field separator.
                  #
                  [[:space:]]{2,10}
               )
            )

            ([\[]S:)                                                # capture group 6

            (                                                       # capture group 7 (start of)
            [[:digit:]]{4}\-[[:digit:]]{1,2}\-[[:digit:]]{1,2}[[:space:]]{1,10}[[:digit:]]{1,2}:[[:digit:]]{1,2}  # 10 provides bound
            )(
             :[[:digit:]]{1,2}
             [\]]
             [[:space:]]{1,500}  # 500 provides bound
             )(
            [[:digit:]]{4}\-[[:digit:]]{1,2}\-[[:digit:]]{1,2}[[:space:]]{1,10}[[:digit:]]{1,2}:[[:digit:]]{1,2}  # 10 provides bound
            )(
            :[[:digit:]]{1,2}[[:space:]]{1,10}  # 10 provides bound
            )

            [\[]LIST:
            ([[:space:]]*[^\]]{1,50})  # 50 provides bound
            [\]]
            (
            [[:space:]]{1,500}.*)  # 500 provides bound
        "## ).unwrap();
}

// FIXME: Get this from a config file or something; maybe find a way to have mutt hand us the list of 'alternate' names?
const IGNORABLE_NONLISTS: [&'static str; 2] = [ "INBOX", "ads" ];  // '


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

// // DEBUG go
//     eprintln!("DEBUG: caps: {:#?}", caps);
// // DEBUG end

    let whatev1  = &caps[1];

    // 2 (3 4 5)  -- 2 is just for debug grouping
    //               3 and 5 are both whitespace
    //               4 has our data (%M or %e/%E expando output)
    let tcnm_lft  = &caps[3];
    let thread_cnt_or_n_of_m = match &caps[4] {  // ex: "17" OR "3/5"
        "1/1" => "   ",   // uninteresting value (clutters display), so we hide it
        val   => val,
    };
    let tcnm_rit  = &caps[5];

    let whatev2  = &caps[6];    // "[S:"

    let dt_lft  = &caps[7];

    let whatev3 = &caps[8];     // seconds portion of left-hand date, closing bracket, plus spaces

    let dt_rit  = &caps[9];

    let whatev4 = &caps[10];     // seconds portion of right-hand date, closing bracket, plus spaces

    let listnm_raw = &caps[11];  // list name (or mail file name, such as 'ads'), possibly surrounded by whitespace

    let whatev5 = &caps[12];     // the rest of the index format line

    let mut outp_string = String::with_capacity( orig_string.len() );

    if dt_lft == dt_rit
    {
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

        // Note that position parameters to format! cannot come after named
        // parameters, hence the ordering...
        outp_string.push_str( &format!( "{}{}{}{}{:width$}{}{}", whatev1,
                                        tcnm_lft, thread_cnt_or_n_of_m, tcnm_rit,
                                        "", dt_rit, whatev4,
                                        width=count_of_needed_spaces ) );
    }
    else
    {
        // The dates are different, so "pass through" date values unchanged

        if BE_VERBOSE {
            eprintln!( "{} (info): dates are different; passing through unchanged", PROG );
        }

        // Reconstruct original input string through the right-hand date portion.
        //
        outp_string.push_str( whatev1 );

        outp_string.push_str( tcnm_lft );
        outp_string.push_str( thread_cnt_or_n_of_m );
        outp_string.push_str( tcnm_rit );

        outp_string.push_str( whatev2 );
        outp_string.push_str( dt_lft  );
        outp_string.push_str( whatev3 );
        outp_string.push_str( dt_rit  );
        outp_string.push_str( whatev4 );

    }

// // DEBUG go
//     eprintln!( "O:{}", orig_string );
//     eprintln!( "N:{}", outp_string );
// // DEBUG end

    // The number of spaces that the list name field should take in total, including
    // the '[' and ']' brackets that we omitted in our regex capturing above (hence
    // the plus two here).
    //
    let needed_listnm_spaces = 2 + UnicodeSegmentation::graphemes( listnm_raw,
                                                                   true /* extended grapheme clusters? (as opposed to "legacy grapheme clusters") */ )
                                   .count();

    let listnm: &str = listnm_raw.trim();

    let mut keep_listnm = true;
    for &one_nonlist in &IGNORABLE_NONLISTS {
        if one_nonlist == listnm {
            keep_listnm = false;
            break
        }
    }

    if true == keep_listnm {

        // Even if we keep the list name, we reformat the field to make the closing
        // right-hand bracket come right after the list name value, but we still
        // append the appropriate number of trailing SPACE characters to use
        // whatever field length we observed with our input, minus the length of the
        // 'LIST:' landmark token.
        //
        //     O:23666  N    [S:2015-10-26 12:55:52]  2015-10-26 12:55:52  [LIST: ads             ]  sender@example.com             (b:   1.3K; l:   144)     blah blah blah some random subject
        //     N:23666  N                             2015-10-26 12:55:52  [ads]                sender@example.com             (b:   1.3K; l:   144)     blah blah blah some random subject
        //
        // remaining_listnm_spaces := ( needed_listnm_spaces - len( listnm ) - 2 )  // minus 2 because we are supplying the '[' and ']' brackets here
        let remaining_listnm_spaces = needed_listnm_spaces
                                      - UnicodeSegmentation::graphemes( listnm,
                                                                        true /* extended grapheme clusters? (as opposed to "legacy grapheme clusters") */ )
                                        .count()
                                      - 2;  // minus 2 because we are supplying the '[' and ']' brackets here

        outp_string.push_str( &format!( "[{}]{:width$}", listnm, "", width=remaining_listnm_spaces ));
    }
    else
    {
        // Here we are suppressing the value in the list name field. We do this
        // simply by emitting the appropriate number of SPACE characters.
        //
        //     O:23666  N    [S:2015-10-26 12:55:52]  2015-10-26 12:55:52  [LIST: ads             ]  sender@example.com             (b:   1.3K; l:   144)     blah blah blah some random subject
        //     N:23666  N                             2015-10-26 12:55:52                       sender@example.com             (b:   1.3K; l:   144)     blah blah blah some random subject
        //
        outp_string.push_str( &format!( "{:width$}", "", width=needed_listnm_spaces ));
    }

    // Tack back on the remainder of the index format line that does not require any special handling
    outp_string.push_str( whatev5 );

// // DEBUG go
//     eprintln!( "O:{}", orig_string );
//     eprintln!( "N:{}", outp_string );
// // DEBUG end

    println!( "{}", outp_string );
}
