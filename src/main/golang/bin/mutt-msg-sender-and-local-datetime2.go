/*
 * Copyright (c) 2015 Alan D. Salewski <salewski@att.net>
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

// mutt-msg-sender-and-local-datetime2 is a rewrite of the
// 'mutt-msg-sender-and-local-datetime' program in go ("golang"), as both an
// exercise in learning the Go programming language and as an attempt to get
// better performance of an interactive mutt session (which invokes the
// program once for every message in a given mailbox when in the mutt 'index'
// view; that amounts to a noticible lag (a second or so, with the original
// bash script version) when invoked hundreds of times in quick succession
// when using a large display).
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
//     "23666  N    [S:2015-10-26 12:55:52]  2015-10-26 12:55:52  sender@example.com             (   144)     blah blah blah some random subject"
//      1234567890123456789012345678901234567890123456789012345678901234567890...
//
// The string starting with '[S:' is the message date in the sender's local
// time, and the date that follows it is my local time.
//
// If the sender's "YYYY-MM-DD hh:mm" portion is identical to my local time,
// this program will suppress the sender's time chunk from the output. For
// example, the output for the above example would be:
//
//     "23666  N                             2015-10-26 12:55:52  sender@example.com             (   144)     blah blah blah some random subject"
//      1234567890123456789012345678901234567890123456789012345678901234567890...
//
// The idea is that I only want to show the sender's time if it is different
// from my local time, as that will allow me to see at a glance in the mutt
// index which messages have come from people operating in different timezones.
//
// If the input is not in the expected format, then the entire value is printed
// "as is" on stdout. So in the event of unexpected input, the program is an
// elaborate NOOP.
//
// HINT:
//
//     folder-hook . 'set index_format="mutt-msg-sender-and-local-datetime \"%4C  %Z  [S:%d]  %D  %-30.30L (%6l) %?X?%2X&  ?  %s\""|'
//
package main;

import "fmt"
import "os"
import "regexp"
import "unicode/utf8"

const PROG = `mutt-msg-sender-and-local-datetime2`

// As the first (and only) argument to the program, we're expecting a string
// that matches this pattern. If there is no first arument provided by the
// caller, or the first parameter does not match this pattern, then we simply
// echo the arguments back to stdout.
//
var RE_EXPECTED_PATTERN = regexp.MustCompile( `` +
        `^(.*[[:space:]]{1,})([[]S:)(` +
        `[[:digit:]]{4}-[[:digit:]]{1,2}-[[:digit:]]{1,2}[[:space:]]{1,}[[:digit:]]{1,2}:[[:digit:]]{1,2}` +
        `)(`  +
        `:[[:digit:]]{1,2}` +
        `[]]` +
        `[[:space:]]{1,}`   +
        `)(`  +
        `[[:digit:]]{4}-[[:digit:]]{1,2}-[[:digit:]]{1,2}[[:space:]]{1,}[[:digit:]]{1,2}:[[:digit:]]{1,2}` +
        `)(`  +
        `:[[:digit:]]{1,2}` +
        `.*)$` );


func main() {

        orig_string := os.Args[1]

        match := RE_EXPECTED_PATTERN.FindStringSubmatch( orig_string )

        // if nil == match {
        if 0 == len( match ) {
                // fmt.Println( `ALJUNK (orig): ` + orig_string )
                fmt.Println( orig_string )
        } else {
                whatev1 := match[1]
                whatev2 := match[2]  // "[S:"

                dt_lft  := match[3]

                whatev3 := match[4]  // "]  "

                dt_rit  := match[5]

                whatev4 := match[6]

                if dt_lft == dt_rit {

                        str_with_olength := whatev2 + dt_lft + whatev3  // full string segment we'll be replacing

                        count_of_needed_spaces := utf8.RuneCountInString( str_with_olength )

                        new_string := fmt.Sprintf( `%s%*s%s%s`, whatev1, count_of_needed_spaces, ``, dt_rit, whatev4 )

// // DEBUG go
//                         fmt.Printf( "O:%s\n", orig_string )
//                         fmt.Printf( "N:%s\n", new_string  )
// // DEBUG end

                        fmt.Println( new_string )


                } else {
                        // The dates are different, so "pass through" full value unchanged
                        fmt.Println( orig_string )
                }
        }
}
