#![allow(non_snake_case)]  // otherwise the compiler complains about the 'WSP' rule.

extern crate pest;

use pest::prelude::*;

impl_rdp! {
  grammar! {
    // This grammar is based on Henrik Norbeck's ABNF grammar for ABC v2.0, with:
    //  1) corrections for its mistakes (e.g. rests could not be generated),
    //  2) rearrangingment of the rules necessary for PEG ordered-choice parsing, and
    //  3) changes to make it more compatible with the ABC v2.1 specification.
    // Norbeck's grammar seems to have disappeared from its original location on the web, but
    // has recently been available at:
    //    https://web.archive.org/web/20120528143746/http://www.norbeck.nu/abc/bnf/abc20bnf.txt

    // The following is roughly ordered by section of the v2.1 specification. It assumes UTF-8
    // encoding.
    music_code_line = { abc_line ~ eoi }

    abc_line = { ( ( barline? ~ element+ ~ (barline ~ element+)* ~ barline? ) | barline ) ~
                 abc_eol }

    element = { broken_rhythm | stem | WSP | chord_or_text | gracing | grace_notes | tuplet |
                slur_begin | slur_end | rollback | multi_measure_rest | measure_repeat |
                nth_repeat | end_nth_repeat | inline_field | hard_line_break | unused_char }

    chord_or_text = { ["\""] ~ (chord | text_expression) ~
                      (chord_newline ~ (chord | text_expression))* ~ ["\""] }
    gracing = { ["."] | userdef_symbol | long_gracing }
    note = { pitch ~ note_length? ~ tie? }
    unused_char = { reserved_char | backquote }  // -FIX- Norbeck included '+', should we?

    // ==== 3.1.6 M: - meter

    meter = { meter_num | ( (["C"] | ["c"]) ~ ["|"]? ) | ["none"] }
    meter_num = { ( ( ["("] ~ WSP* ~ DIGITS ~ (WSP* ~ ["+"] ~ WSP* ~ DIGITS)* ~ WSP* ~ [")"] ) |
                    ( DIGITS ~ (WSP* ~ ["+"] ~ WSP* ~ DIGITS)* ) ) ~
                  WSP* ~ ["/"] ~ WSP* ~ DIGITS }

    // ==== 3.1.8 Q: - tempo

    tempo = { ( tempo_spec ~ ( WSP+ ~ tempo_desc )? ) | ( tempo_desc ~ ( WSP+ ~ tempo_spec )? ) }
    tempo_spec = { ( note_length_strict ~ ["="] ~ DIGITS ) |
                   ( (["C"] | ["c"]) ~ note_length? ~ ["="] ~ DIGITS ) | DIGITS }
    tempo_desc = { ["\""] ~ non_quote* ~ ["\""] }

    // ==== 3.1.14 K: - key

    key = { ( key_def ~ ( WSP+ ~ clef )? ) | clef | ["HP"] | ["Hp"] }
    key_def = { basenote ~ (["#"] | ["b"] | ["♯"] | ["♭"])? ~ ( WSP* ~ mode )? ~
                ( WSP ~ ( WSP* ~ global_accidental )+ )* }
    mode = { major | lydian | ionian | mixolydian | dorian | aeolian | phrygian | locrian | minor |
             ["exp"] }
    major = { ["maj"] ~ (["o"] ~ ["r"]?)? }
    lydian = { ["lyd"] ~ (["i"] ~ (["a"] ~ (["n"])?)?)? }
    ionian = { ["ion"] ~ (["i"] ~ (["a"] ~ (["n"])?)?)? }
    mixolydian = { ["mix"] ~ (["o"] ~ (["l"] ~ (["y"] ~ (["d"] ~ (["i"] ~ (["a"] ~ (["n"])?)?)?)?)?)?)? }
    dorian = { ["dor"] ~ (["i"] ~ (["a"] ~ (["n"])?)?)? }
    aeolian = { ["aeo"] ~ (["l"] ~ (["i"] ~ (["a"] ~ (["n"])?)?)?)? }
    phrygian = { ["phr"] ~ (["y"] ~ (["g"] ~ (["i"] ~ (["a"] ~ (["n"])?)?)?)?)? }
    locrian = { ["loc"] ~ (["r"] ~ (["i"] ~ (["a"] ~ (["n"])?)?)?)? }
    minor = { ["m"] ~ (["in"] ~ (["o"] ~ ["r"]?)?)? }
    global_accidental = { accidental ~ basenote }

    // ==== 3.2 Use of fields within the tune body

    inline_field = { ifield_text | ifield_key | ifield_length | ifield_meter | ifield_part |
                     ifield_tempo | ifield_userdef | ifield_voice }
    ifield_text = { ["["] ~ (["I"] | ["N"] | ["R"] | ["r"]) ~ [":"] ~ non_right_bracket+ ~ ["]"] }
    ifield_key = { ["[K:"] ~ WSP* ~ ( ["none"] | key? ) ~ ["]"] }
    ifield_length = { ["[L:"] ~ WSP* ~ note_length_strict ~ ["]"] }
    ifield_meter = { ["[M:"] ~ WSP* ~ meter ~ ["]"] }
    // -FIX- define ABC2.1 'm:' macro field
    // ifield_part: 'P:' fields are supposed to be very structured (see 3.1.9), but in the wild, they
    // are frequently abused. Accept any non-']' text:
    ifield_part = { ["[P:"] ~ non_right_bracket+ ~  ["]"] }
    ifield_tempo = { ["[Q:"] ~ WSP* ~ tempo ~ ["]"] }
    ifield_userdef = { ["[U:"] ~ non_right_bracket+ ~ ["]"] }
    ifield_voice = { ["[V:"] ~ WSP* ~ voice ~ ["]"] }

    // ==== 4.1 Pitch

    pitch = { accidental? ~ basenote ~ octave? }
    basenote = { ['A'..'G'] | ['a'..'g'] }
    octave = { ["'"]+ | [","]+ }

    // ==== 4.2 Accidentals

    accidental = { ["^^"] | ["^"] | ["__"] | ["_"] | ["="] }

    // ==== 4.3 Note lengths

    // Norbeck specified this as "(DIGITS? ('/' DIGITS)?) / '/'+", which could match the empty
    // string. We need the note_length parser to fail if it doesn't match anything. Things we
    // need to match include: '2', '/2', '3/2', '/', '//'.
    note_length = { note_length_smaller | note_length_full | note_length_bigger | note_length_slashes }
    note_length_bigger = { DIGITS }  // !FIX! had '+' on the end  # DIGITS is already greedy, but this avoids an optimization (bug?)
    note_length_smaller = { ["/"] ~ DIGITS }
    note_length_full = { DIGITS ~ ["/"] ~ DIGITS }
    note_length_slashes = { ["/"]+ }

    // used by various fields
    note_length_strict = { ( DIGITS ~ ["/"] ~ DIGITS ) | ["1"] }

    // ==== 4.4 Broken rhythm

    broken_rhythm = { stem ~ b_elem* ~ b_sep ~ b_sep? ~ b_sep? ~ b_elem* ~ stem }
    b_sep = { (["<"] | [">"]) }
    b_elem = { WSP | chord_or_text | gracing | grace_notes | slur_begin | slur_end }

    // ==== 4.5 Rests

    rest = { ( ["x"] | ["y"] | ["z"] ) ~ note_length? }
    multi_measure_rest = { ["Z"] ~ ['0'..'9']* }

    // ==== 4.6 Clefs and transposition

    clef = { ( clef_spec | clef_middle | clef_transpose | clef_octave | clef_stafflines ) ~
             ( WSP+ ~ clef )? }
    clef_spec = { ( ( ["clef="] ~ ( clef_note | clef_name ) ) | clef_name ) ~ clef_line? ~
                  ( ["+8"] | ["-8"] )? ~ ( WSP+ ~ clef_middle )? }
    clef_note = { ["G"] | ["C"] | ["F"] | ["P"] }  // non-standard, from Norbeck
    clef_name = { ["treble"] | ["alto"] | ["tenor"] | ["bass"] | ["perc"] | ["none"] }
    clef_line = { ['1'..'5'] }
    clef_middle = { ["middle="] ~ basenote ~ octave? }
    clef_transpose = { ["transpose="] ~ ["-"]? ~ DIGITS }
    clef_octave = { ["octave="] ~ ["-"]? ~ DIGITS }
    clef_stafflines = { ["stafflines="] ~ DIGITS }

    // ==== 4.7 Beams

    backquote = { ["`"] }  // used to increase legibility in groups of beamed notes, otherwise meaningless

    // ==== 4.8 Repeat/bar symbols

    barline = { invisible_barline |
                ( [":"]* ~ ["["]? ~ ( ["."]? ~ ["|"] )+ ~ ( ["]"] | [":"]+ | nth_repeat_num )? ) |
                double_repeat_barline | dashed_barline }
    invisible_barline = { ["[|]"] | ["[]"] }  // second is non-standard, from Norbeck
    double_repeat_barline = { ["::"] }
    dashed_barline = { [":"] }  // non-standard, from Norbeck

    // ==== 4.9 First and second repeats

    nth_repeat = { ["["] ~ ( nth_repeat_num | nth_repeat_text ) }
    nth_repeat_num = { DIGITS ~ ( ( [","] | ["-"] ) ~ DIGITS )* }
    nth_repeat_text = { ["\""] ~ non_quote* ~ ["\""] }  // from Norbeck, not in the standard?
    end_nth_repeat = { ["]"] }

    // ==== 4.10 Variant endings -- see 4.8 Repeat/bar symbols

    // ==== 4.11 Ties and slurs

    // see '4.20 Order of abc constructs' for more on ties
    tie = { ["-"] }
    slur_begin = { ["("] }
    slur_end = { [")"] }

    // ==== 4.12 Grace notes

    // -FIX- Norbeck didn't include broken rhythm here, and I haven't yet implemented it. I have seen
    // it in the wild, though rarely.
    grace_notes = { ["{"] ~ acciaccatura? ~ grace_note_stem+ ~ ["}"] }
    grace_note_stem = { grace_note | ( ["["] ~ grace_note ~ grace_note+ ~ ["]"] ) }  // from Norbeck; non-standard extension
    grace_note = { pitch ~ note_length? }
    acciaccatura = { ["/"] }

    // ==== 4.13 Duplets, triplets, quadruplets, etc.

    // Norbeck included two or more elements as part of the tuplet, but here we'd need to tell the
    // parser to match as many elements as the value of the first DIGITS.
    tuplet = { ["("] ~ DIGITS ~ ( [":"] ~ DIGITS? ~ [":"] ~ DIGITS? )? }

    // ==== 4.14 Decorations

    // -FIX- 'I:decoration +' could change '!' to '+'
    long_gracing = { ( ["!"] ~ ( gracing1 | gracing2 | gracing3 | gracing_nonstandard | gracing4 ) ~
                       ["!"] ) |
                     ( ["!"] ~ gracing_catchall ~ ["!"] ) }
    gracing1 = { ["<("] | ["<)"] | [">("] | [">)"] | ["D.C."] | ["D.S."] | ["accent"] |
                 ["arpeggio"] | ["breath"] | ["coda"] | ["crescendo("] | ["crescendo)"] |
                 ["dacapo"] | ["dacoda"] | ["diminuendo("] }
    gracing2 = { ["diminuendo)"] | ["downbow"] | ["emphasis"] | ["fermata"] | ["ffff"] | ["fff"] |
                 ["ff"] | ["fine"] | ["invertedfermata"] | ["invertedturnx"] | ["invertedturn"] |
                 ["longphrase"] | ["lowermordent"] }
    gracing3 = { ["mediumphrase"] | ["mf"] | ["mordent"] | ["mp"] | ["open"] | ["plus"] | ["pppp"] |
                 ["ppp"] | ["pp"] | ["pralltriller"] | ["roll"] | ["segno"] | ["sfz"] |
                 ["shortphrase"] | ["slide"] | ["snap"] }
    gracing4 = { ["tenuto"] | ["thumb"] | ["trill("] | ["trill)"] | ["trill"] | ["turnx"] |
                 ["turn"] | ["upbow"] | ["uppermordent"] | ["wedge"] | ["+"] | ['0'..'5'] | ["<"] |
                 [">"] | ["f"] | ["p"] }
    gracing_nonstandard = { ["cresc"] | ["decresc"] | ["dimin"] | ["fp"] |
                            ( ["repeatbar"] ~ DIGITS ) }  // non-standard, from Norbeck
    gracing_catchall = { ['"'..'~']+ }  // catch-all for non-standard ABC

    // ==== 4.16 Redefinable symbols

    userdef_symbol = { ["~"] | ['H'..'Y'] | ['h'..'w'] }  // Norbeck includes non-standard 'X' and 'Y'

    // ==== 4.17 Chords and unisons

    // Norbeck used "chord" for chord symbols, and "stem" for what the spec calls chords.
    stem = { ( ["["] ~ note ~ note+ ~ ["]"] ~ tie? ) | note | rest }

    // ==== 4.18 Chord symbols

    // 'non_quote' is a catch-all for non-conforming ABC (in practice, people sometimes confuse the
    // chord symbol and annotation syntaxes.) Norbeck's grammar let it eat everything else between
    // the quotes; here we use a negative lookahead assert to make sure it doesn't eat a
    // chord_newline.
    chord = { basenote ~ chord_accidental? ~ chord_type? ~ (["/"] ~ basenote ~ chord_accidental?)? ~
              (!chord_newline ~ non_quote)*
            }

    // the last three here are \\u266f sharp symbol, \\u266d flat symbol, and \\u266e natural symbol
    chord_accidental = { ["#"] | ["b"] | ["="] | ["♯"] | ["♭"] | ["♮"] }

    // chord type, e.g. m, min, maj7, dim, sus4: "programs should treat chord symbols quite liberally"
    chord_type = { (['A'..'Z'] | ['a'..'z'] | ['0'..'9']+ | ["-"])+ }

    // ==== 4.19 Annotations

    text_expression = { ( ( ["^"] | ["<"] | [">"] | ["_"] | ["@"] ) ~
                          (!chord_newline ~ non_quote)+ ) |
                        bad_text_expression
                      }
    bad_text_expression = { (!chord_newline ~ non_quote)+ }  // no leading placement symbol

    // ==== 6.1.1 Typesetting linebreaks

    // this would include comments, if we did not strip them already:
    abc_eol = { line_continuation? ~ WSP* }
    line_continuation = { ["\\"] }

    // -FIX- this could be changed by a 'I:linebreak' field:
    hard_line_break = { ["$"] | ["!"] }

    // ==== 7. Multiple voices

    voice = { (!([" "] | ["]"]) ~ any)+ ~
              ( WSP+ ~ (!([" "] | ["="] | ["]"]) ~ any)+ ~ ["="] ~
                ( ( ["\""] ~ non_quote* ~ ["\""] ) | (!([" "] | ["]"]) ~ any)+ ) )* }

    // ==== 7.4 Voice overlay

    rollback = { ["&"] }

    // ==== 8.1 Tune body

    reserved_char = { ["#"] | ["*"] | [";"] | ["?"] | ["@"] }

    // ==== utility rules

    chord_newline = { ["\\n"] | [";"] }  // from Norbeck; non-standard extension
    measure_repeat = { ["/"] ~ ["/"]? }  // from Norbeck; non-standard extension
    non_quote = { !["\""] ~ any }
    non_right_bracket = { !["]"] ~ any }
    DIGITS = { ['0'..'9']+ }
    WSP = { ([" "] | ["\t"])+ }  // whitespace
  }
}
