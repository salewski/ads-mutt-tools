/*
 * Copyright (c) 2015, 2019 Alan D. Salewski <salewski@att.net>
 *
 *     This program is free software; you can redistribute it and/or modify
 *     it under the terms of the GNU General Public License as published by
 *     the Free Software Foundation; either version 2 of the License, or
 *     (at your option) any later version.
 *
 *     This program is distributed in the hope that it will be useful,
 *     but WITHOUT ANY WARRANTY; without even the implied warranty of
 *     MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *     GNU General Public License for more details.
 *
 *     You should have received a copy of the GNU General Public License
 *     along with this program; if not, write to the Free Software Foundation,
 *     Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301,, USA.
 */

// ads-mutt-index-format-custom2 is a rewrite of the
// 'ads-mutt-index-format-custom' program in go ("golang"), but is derived
// from both that bash script and the earlier
// 'mutt-msg-sender-and-local-datetime2' golang program.
//
// Similar to the motivation of 'mutt-msg-sender-and-local-datetime2', the
// functionality of 'ads-mutt-index-format-custom2' was first prototyped in
// the bash program 'ads-mutt-index-format-custom', and is being recast here
// in golang to achieve better performance of an interactive mutt session
// (which invokes the program once for every message in a given mailbox when
// in the mutt 'index' view; that amounts to a noticible lag (upwards of 5 or
// 6 seconds with the original bash script version) when invoked hundreds of
// times in quick succession when using a large display).
//
// This program is intended to be invoked from mutt at runtime (via a format
// string that ends with a '|' character; see the section "Filters" in the
// mutt manual). The output from this program will be used by mutt as the
// updated filter string, which mutt will parse again.
//
// This filter expects to receive (as the first non-option parameter) the
// formatted string for the mutt 'index' view on stdin, which is expected to
// contain two date fields (the message date in the sender's local time, and my
// local time), delineated with something like this:
//
//     "23666  N    [S:2015-10-26 12:55:52]  2015-10-26 12:55:52  [LIST: ads             ]  sender@example.com             (b:   1.3K; l:   144)     blah blah blah some random subject"
//      1234567890123456789012345678901234567890123456789012345678901234567890...
//
// The string starting with '[S:' is the message date in the sender's local
// time, and the date that follows it is my local time.
//
// The string starting with '[LIST:' is the "username" portion of a known AND
// subscribed mailing list address, or the name of the mailbox file if the message
// is not to such a list (this is the behavior of mutt's '%B' expando).
//
// If the sender's "YYYY-MM-DD hh:mm" portion is identical to my local time
// (accurate to the minute; the seconds field is ignored), this program will
// suppress the sender's time chunk from the output. For example, the output
// for the above example would be:
//
//     "23666  N                             2015-10-26 12:55:52  [ads]                sender@example.com             (b:  1.3K; l:   144)     blah blah blah some random subject"
//      1234567890123456789012345678901234567890123456789012345678901234567890...
//
// The idea is that I only want to show the sender's time if it is different
// from my local time, as that will allow me to see at a glance in the mutt
// index which messages have come from people operating in different timezones.
//
// Similarly for mailing list names: I do not want to see '[ads]' as the mailing
// list name a gazillion times in my inbox, so this program will remove the field
// from the output if the provides value matches one of the preconfigured known
// values;  see '__known_alternates' below.
//
// FIXME: Maybe figure out a way to get mutt to pass the list of 'alternate' values here?
//
// If the input is not in the expected format, then the entire value is printed
// "as is" on stdout. So in the event of unexpected input, the program is an
// elaborate NOOP.
//
// HINT:
//
//     folder-hook . 'set index_format="ads-mutt-index-format-custom2 \"%4C  %Z  [S:%d]  %D  [LIST: %-16.16B]  %-30.30F (b: %6c; l: %6l) %?X?%2X&  ?  %s\""|'
//
package main;

import "fmt"
import "os"
import "regexp"
import "strings"
import "unicode/utf8"

const _PROG = `ads-mutt-index-format-custom2`

const _BE_VERBOSE = false
// const _BE_VERBOSE = true


// As the first (and only) argument to the program, we're expecting a string
// that matches this pattern. If there is no first arument provided by the
// caller, or the first parameter does not match this pattern, then we simply
// echo the arguments back to stdout.
//
// Note that we do not capture the '[LIST:' tag or its closing ']' bracket. The
// brackets are re-inserted down below, and the 'LIST:' token is omitted in the
// output (it is only used by our mutt config to create an easy-to-use landmark
// for this program).
//
// Example input strings (each indented here by 4 spaces for readability):
//
//     5  N F  [S:2005-06-02 11:52:28]  2005-06-02 11:52:28  [LIST: ads             ]  To asalewski@gmail.com         (b:   0.1K; l:      1)     testing 01
//
var _RE_EXPECTED_PATTERN = regexp.MustCompile( `` +

    `^([[:space:]]*[[:digit:]]{1,}[[:space:]]{1,}[^[]{1,})` +  // capture group 1
    `([[]S:)` +                                                // capture group 2
    `(` +                                                      // capture group 3 (start of)

    `[[:digit:]]{4}-[[:digit:]]{1,2}-[[:digit:]]{1,2}[[:space:]]{1,}[[:digit:]]{1,2}:[[:digit:]]{1,2}` +
    `)(` +
    `:[[:digit:]]{1,2}` +
    `[]]` +
    `[[:space:]]{1,}` +
    `)(` +
    `[[:digit:]]{4}-[[:digit:]]{1,2}-[[:digit:]]{1,2}[[:space:]]{1,}[[:digit:]]{1,2}:[[:digit:]]{1,2}` +
    `)(` +
    `:[[:digit:]]{1,2}[[:space:]]{1,}` +
    `)` +
    `[[]LIST:` +
    `([[:space:]]*[^]]{1,})` +
    `[]]` +
    `(` +
    `[[:space:]]{1,}.*)` );


// FIXME: Get this from a config file or something; maybe find a way to have mutt hand us the list of 'alternate' names?
var gl_ignorable_nonlists = [...]string{ `ads` }


func main() {

// // DEBUG go
//     if _BE_VERBOSE {
//         fmt.Fprintf( os.Stderr, "%s (info): starting\n", _PROG )
//         fmt.Fprintf( os.Stderr, "%s (info): len of os.Args: %d\n", _PROG, len( os.Args ) )
//     }
// // DEBUG end

    if 2 < len( os.Args ) {
        fmt.Fprintf( os.Stderr, "%s (error): required mutt pager_format line not provided; bailing out\n", _PROG )
        os.Exit(1)
    }

    var orig_string string = os.Args[1]
    var outp_string string = ``

    match := _RE_EXPECTED_PATTERN.FindStringSubmatch( orig_string )

    // if nil == match {
    if 0 == len( match ) {

        fmt.Fprintf( os.Stderr, "%s (warning): input line did not match regex; passing through unchanged\n", _PROG )

        fmt.Println( orig_string )
        return
    }

    if _BE_VERBOSE {
        fmt.Fprintf( os.Stderr, "%s (info): input line matched regex\n", _PROG )
    }


    whatev1 := match[1]
    whatev2 := match[2]     // "[S:"

    dt_lft  := match[3]

    whatev3 := match[4]     // seconds portion of left-hand date, closing bracket, plus spaces

    dt_rit  := match[5]

    whatev4 := match[6]     // seconds portion of right-hand date, closing bracket, plus spaces

    listnm_raw := match[7]  // list name (or mail file name, such as 'ads'), possibly surrounded by whitespace

    whatev5 := match[8]     // the rest of the index format line


    if dt_lft == dt_rit {

        str_with_olength := whatev2 + dt_lft + whatev3  // full string segment we'll be replacing

        count_of_needed_spaces := utf8.RuneCountInString( str_with_olength )

        outp_string += fmt.Sprintf( `%s%*s%s%s`, whatev1, count_of_needed_spaces, ``, dt_rit, whatev4 )

// // DEBUG go
//         fmt.Printf( "O:%s\n", orig_string )
//         fmt.Printf( "N:%s\n", outp_string  )
// // DEBUG end
    } else {

        // The dates are different, so "pass through" date values unchanged

        if _BE_VERBOSE {
            fmt.Fprintf( os.Stderr, "%s (info): dates are different; passing through unchanged\n", _PROG )
        }

        // Reconstruct original input string through the right-hand date portion.
        outp_string += fmt.Sprintf( `%s%s%s%s%s%s`, whatev1, whatev2, dt_lft, whatev3, dt_rit, whatev4 )

        // fmt.Println( orig_string )
    }

    // The number of spaces that the list name field should take in total, including
    // the '[' and ']' brackets that we omitted in our regex capturing above (hence
    // the plus two here).
    //
    needed_listnm_spaces := ( 2 + utf8.RuneCountInString( listnm_raw ) )

    listnm := strings.TrimSpace( listnm_raw )

    keep_listnm := true
    for _, one_nonlist := range gl_ignorable_nonlists {
        if one_nonlist == listnm {
            keep_listnm = false
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
        remaining_listnm_spaces := ( needed_listnm_spaces - len( listnm ) - 2 )  // minus 2 because we are supplying the '[' and ']' brackets here

        outp_string += fmt.Sprintf( `[%s]%*s`, listnm, remaining_listnm_spaces, `` )

    } else {

        // Here we are suppressing the value in the list name field. We do this
        // simply by emitting the appropriate number of SPACE characters.
        //
        //     O:23666  N    [S:2015-10-26 12:55:52]  2015-10-26 12:55:52  [LIST: ads             ]  sender@example.com             (b:   1.3K; l:   144)     blah blah blah some random subject
        //     N:23666  N                             2015-10-26 12:55:52                       sender@example.com             (b:   1.3K; l:   144)     blah blah blah some random subject
        //
        outp_string += fmt.Sprintf( `%*s`, needed_listnm_spaces, `` )
    }

    // Tack back on the remainder of the index format line that does not require any special handling
    outp_string += fmt.Sprintf( `%s`, whatev5 )

// // DEBUG go
//     fmt.Fprintf( os.Stderr, "O:%s\n", orig_string )
//     fmt.Fprintf( os.Stderr, "N:%s\n", outp_string )
// // DEBUG end

    fmt.Println( outp_string )
}
