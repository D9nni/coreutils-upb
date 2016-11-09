extern crate unindent;

use common::util::*;
use std::path::Path;
use std::env;
use std::io::Write;
use std::fs::File;
use std::fs::remove_file;
use self::unindent::*;


// octal dump of 'abcdefghijklmnopqrstuvwxyz\n'
static ALPHA_OUT: &'static str = "
        0000000 061141 062143 063145 064147 065151 066153 067155 070157
        0000020 071161 072163 073165 074167 075171 000012
        0000033
        ";

// XXX We could do a better job of ensuring that we have a fresh temp dir to ourself,
// not a general one ful of other proc's leftovers.

// Test that od can read one file and dump with default format
#[test]
fn test_file() {
    use std::env;
    let temp = env::temp_dir();
    let tmpdir = Path::new(&temp);
    let file = tmpdir.join("test");

    {
        let mut f = File::create(&file).unwrap();
        match f.write_all(b"abcdefghijklmnopqrstuvwxyz\n") {
            Err(_)  => panic!("Test setup failed - could not write file"),
            _ => {}
        }
    }

    let result = new_ucmd!().arg("--endian=little").arg(file.as_os_str()).run();

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, unindent(ALPHA_OUT));

    let _ = remove_file(file);
}

// Test that od can read 2 files and concatenate the contents
#[test]
fn test_2files() {
    let temp = env::temp_dir();
    let tmpdir = Path::new(&temp);
    let file1 = tmpdir.join("test1");
    let file2 = tmpdir.join("test2");

    for &(n,a) in [(1,"a"), (2,"b")].iter() {
        println!("number: {} letter:{}", n, a);
     }

    for &(path,data)in &[(&file1, "abcdefghijklmnop"),(&file2, "qrstuvwxyz\n")] {
        let mut f = File::create(&path).unwrap();
        match f.write_all(data.as_bytes()) {
            Err(_)  => panic!("Test setup failed - could not write file"),
            _ => {}
        }
    }

    let result = new_ucmd!().arg("--endian=little").arg(file1.as_os_str()).arg(file2.as_os_str()).run();

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, unindent(ALPHA_OUT));

    let _ = remove_file(file1);
    let _ = remove_file(file2);
}

// Test that od gives non-0 exit val for filename that dosen't exist.
#[test]
fn test_no_file() {
    let temp = env::temp_dir();
    let tmpdir = Path::new(&temp);
    let file = tmpdir.join("}surely'none'would'thus'a'file'name");

    let result = new_ucmd!().arg(file.as_os_str()).run();

    assert!(!result.success);
}

// Test that od reads from stdin instead of a file
#[test]
fn test_from_stdin() {
    let input = "abcdefghijklmnopqrstuvwxyz\n";
    let result = new_ucmd!().arg("--endian=little").run_piped_stdin(input.as_bytes());

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, unindent(ALPHA_OUT));
}

// Test that od reads from stdin and also from files
#[test]
fn test_from_mixed() {
    let temp = env::temp_dir();
    let tmpdir = Path::new(&temp);
    let file1 = tmpdir.join("test-1");
    let file3 = tmpdir.join("test-3");

    let (data1, data2, data3) = ("abcdefg","hijklmnop","qrstuvwxyz\n");
    for &(path,data)in &[(&file1, data1),(&file3, data3)] {
        let mut f = File::create(&path).unwrap();
        match f.write_all(data.as_bytes()) {
            Err(_)  => panic!("Test setup failed - could not write file"),
            _ => {}
        }
    }

    let result = new_ucmd!().arg("--endian=little").arg(file1.as_os_str()).arg("-").arg(file3.as_os_str()).run_piped_stdin(data2.as_bytes());

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, unindent(ALPHA_OUT));
}

#[test]
fn test_multiple_formats() {
    let input = "abcdefghijklmnopqrstuvwxyz\n";
    let result = new_ucmd!().arg("-c").arg("-b").run_piped_stdin(input.as_bytes());

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, unindent("
            0000000   a   b   c   d   e   f   g   h   i   j   k   l   m   n   o   p
                    141 142 143 144 145 146 147 150 151 152 153 154 155 156 157 160
            0000020   q   r   s   t   u   v   w   x   y   z  \\n
                    161 162 163 164 165 166 167 170 171 172 012
            0000033
            "));
}

#[test]
fn test_dec() {
    let input = [
    	0u8, 0u8,
    	1u8, 0u8,
    	2u8, 0u8,
    	3u8, 0u8,
    	0xffu8,0x7fu8,
    	0x00u8,0x80u8,
    	0x01u8,0x80u8,];
    let expected_output = unindent("
            0000000      0      1      2      3  32767 -32768 -32767
            0000016
            ");
    let result = new_ucmd!().arg("--endian=little").arg("-s").run_piped_stdin(&input[..]);

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, expected_output);
}

#[test]
fn test_hex16(){
    let input: [u8; 9] = [
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xff];
    let expected_output = unindent("
            0000000 2301 6745 ab89 efcd 00ff
            0000011
            ");
    let result = new_ucmd!().arg("--endian=little").arg("-x").run_piped_stdin(&input[..]);

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, expected_output);
}

#[test]
fn test_hex32(){
    let input: [u8; 9] = [
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xff];
    let expected_output = unindent("
            0000000 67452301 efcdab89 000000ff
            0000011
            ");
    let result = new_ucmd!().arg("--endian=little").arg("-X").run_piped_stdin(&input[..]);

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, expected_output);
}

#[test]
fn test_f16(){
    let input: [u8; 14] = [
        0x00, 0x3c, // 0x3C00 1.0
        0x00, 0x00, // 0x0000 0.0
        0x00, 0x80, // 0x8000 -0.0
        0x00, 0x7c, // 0x7C00 Inf
        0x00, 0xfc, // 0xFC00 -Inf
        0x00, 0xfe, // 0xFE00 NaN
        0x00, 0x84];// 0x8400 -6.104e-5
    let expected_output = unindent("
            0000000     1.000         0        -0       inf
            0000010      -inf       NaN -6.104e-5
            0000016
            ");
    let result = new_ucmd!().arg("--endian=little").arg("-tf2").arg("-w8").run_piped_stdin(&input[..]);

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, expected_output);
}

#[test]
fn test_f32(){
    let input: [u8; 28] = [
        0x52, 0x06, 0x9e, 0xbf, // 0xbf9e0652 -1.2345679
        0x4e, 0x61, 0x3c, 0x4b, // 0x4b3c614e 12345678
        0x0f, 0x9b, 0x94, 0xfe, // 0xfe949b0f -9.876543E37
        0x00, 0x00, 0x00, 0x80, // 0x80000000 -0.0
        0xff, 0xff, 0xff, 0x7f, // 0x7fffffff NaN
        0xc2, 0x16, 0x01, 0x00, // 0x000116c2 1e-40
        0x00, 0x00, 0x7f, 0x80];// 0x807f0000 -1.1663108E-38
    let expected_output = unindent("
            0000000     -1.2345679       12345678  -9.8765427e37             -0
            0000020            NaN          1e-40 -1.1663108e-38
            0000034
            ");
    let result = new_ucmd!().arg("--endian=little").arg("-f").run_piped_stdin(&input[..]);

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, expected_output);
}

#[test]
fn test_f64(){
    let input: [u8; 40] = [
        0x27, 0x6b, 0x0a, 0x2f, 0x2a, 0xee, 0x45, 0x43, // 0x4345EE2A2F0A6B27 12345678912345678
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0x0000000000000000 0
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x80, // 0x8010000000000000 -2.2250738585072014e-308
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0x0000000000000001 5e-324 (subnormal)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0];// 0xc000000000000000 -2
    let expected_output = unindent("
            0000000        12345678912345678                        0
            0000020 -2.2250738585072014e-308                   5e-324
            0000040      -2.0000000000000000
            0000050
            ");
    let result = new_ucmd!().arg("--endian=little").arg("-F").run_piped_stdin(&input[..]);

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, expected_output);
}

#[test]
fn test_multibyte() {
    let result = new_ucmd!().arg("-c").arg("-w12").run_piped_stdin("Universität Tübingen \u{1B000}".as_bytes());

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, unindent("
            0000000   U   n   i   v   e   r   s   i   t   ä  **   t
            0000014       T   ü  **   b   i   n   g   e   n       \u{1B000}
            0000030  **  **  **
            0000033
            "));
}

#[test]
fn test_width(){
    let input: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let expected_output = unindent("
            0000000 000000 000000
            0000004 000000 000000
            0000010
            ");

    let result = new_ucmd!().arg("-w4").arg("-v").run_piped_stdin(&input[..]);

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, expected_output);
}

#[test]
fn test_invalid_width(){
    let input: [u8; 4] = [0x00, 0x00, 0x00, 0x00];
    let expected_output = unindent("
            0000000 000000
            0000002 000000
            0000004
            ");

    let result = new_ucmd!().arg("-w5").arg("-v").run_piped_stdin(&input[..]);

    assert_eq!(result.stderr, "od: warning: invalid width 5; using 2 instead\n");
    assert!(result.success);
    assert_eq!(result.stdout, expected_output);
}

#[test]
fn test_zero_width(){
    let input: [u8; 4] = [0x00, 0x00, 0x00, 0x00];
    let expected_output = unindent("
            0000000 000000
            0000002 000000
            0000004
            ");

    let result = new_ucmd!().arg("-w0").arg("-v").run_piped_stdin(&input[..]);

    assert_eq!(result.stderr, "od: warning: invalid width 0; using 2 instead\n");
    assert!(result.success);
    assert_eq!(result.stdout, expected_output);
}

#[test]
fn test_width_without_value(){
    let input: [u8; 40] = [0 ; 40];
    let expected_output = unindent("
            0000000 000000 000000 000000 000000 000000 000000 000000 000000 000000 000000 000000 000000 000000 000000 000000 000000
            0000040 000000 000000 000000 000000
            0000050
            ");

    let result = new_ucmd!().arg("-w").run_piped_stdin(&input[..]);

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, expected_output);
}

#[test]
fn test_suppress_duplicates(){
    let input: [u8; 41] =  [
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            1, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0];
    let expected_output = unindent("
            0000000 00000000000
                     0000  0000
            *
            0000020 00000000001
                     0001  0000
            0000024 00000000000
                     0000  0000
            *
            0000050 00000000000
                     0000
            0000051
            ");

    let result = new_ucmd!().arg("-w4").arg("-O").arg("-x").run_piped_stdin(&input[..]);

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, expected_output);
}

#[test]
fn test_big_endian() {
    let input: [u8; 8] = [
        0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];// 0xc000000000000000 -2

    let expected_output = unindent("
        0000000           -2.0000000000000000
                    -2.0000000              0
                      c0000000       00000000
                   c000   0000    0000   0000
        0000010
        ");

    let result = new_ucmd!().arg("--endian=big").arg("-F").arg("-f").arg("-X").arg("-x").run_piped_stdin(&input[..]);

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, expected_output);
}

#[test]
#[allow(non_snake_case)]
fn test_alignment_Xxa() {
    let input: [u8; 8] = [
        0x0A, 0x0D, 0x65, 0x66, 0x67, 0x00, 0x9e, 0x9f];

    let expected_output = unindent("
        0000000        66650d0a        9f9e0067
                   0d0a    6665    0067    9f9e
                 nl  cr   e   f   g nul  rs  us
        0000010
        ");

    // in this case the width of the -a (8-bit) determines the alignment for the other fields
    let result = new_ucmd!().arg("--endian=little").arg("-X").arg("-x").arg("-a").run_piped_stdin(&input[..]);

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, expected_output);
}

#[test]
#[allow(non_snake_case)]
fn test_alignment_Fx() {
    let input: [u8; 8] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0];// 0xc000000000000000 -2

    let expected_output = unindent("
        0000000      -2.0000000000000000
                  0000  0000  0000  c000
        0000010
        ");

    // in this case the width of the -F (64-bit) determines the alignment for the other field
    let result = new_ucmd!().arg("--endian=little").arg("-F").arg("-x").run_piped_stdin(&input[..]);

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, expected_output);
}

#[test]
fn test_maxuint(){
    let input = [0xFFu8 ; 8];
    let expected_output = unindent("
            0000000          1777777777777777777777
                        37777777777     37777777777
                     177777  177777  177777  177777
                    377 377 377 377 377 377 377 377
                               18446744073709551615
                         4294967295      4294967295
                      65535   65535   65535   65535
                    255 255 255 255 255 255 255 255
            0000010
            ");

    let result = new_ucmd!().arg("--format=o8").arg("-Oobtu8").arg("-Dd").arg("--format=u1").run_piped_stdin(&input[..]);

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, expected_output);
}

#[test]
fn test_hex_offset(){
    let input = [0u8 ; 0x1F];
    let expected_output = unindent("
            000000 00000000 00000000 00000000 00000000
                   00000000 00000000 00000000 00000000
            000010 00000000 00000000 00000000 00000000
                   00000000 00000000 00000000 00000000
            00001F
            ");

    let result = new_ucmd!().arg("-Ax").arg("-X").arg("-X").run_piped_stdin(&input[..]);

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, expected_output);
}

#[test]
fn test_dec_offset(){
    let input = [0u8 ; 19];
    let expected_output = unindent("
            0000000 00000000 00000000 00000000 00000000
                    00000000 00000000 00000000 00000000
            0000016 00000000
                    00000000
            0000019
            ");

    let result = new_ucmd!().arg("-Ad").arg("-X").arg("-X").run_piped_stdin(&input[..]);

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, expected_output);
}

#[test]
fn test_no_offset(){
    let input = [0u8 ; 31];
    const LINE: &'static str = " 00000000 00000000 00000000 00000000\n";
    let expected_output = [LINE, LINE, LINE, LINE].join("");

    let result = new_ucmd!().arg("-An").arg("-X").arg("-X").run_piped_stdin(&input[..]);

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, expected_output);
}

#[test]
fn test_invalid_offset(){
    let result = new_ucmd!().arg("-Ab").run();

    assert!(!result.success);
}

#[test]
fn test_skip_bytes(){
    let input = "abcdefghijklmnopq";
    let result = new_ucmd!().arg("-c").arg("--skip-bytes=5").run_piped_stdin(input.as_bytes());

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, unindent("
            0000005   f   g   h   i   j   k   l   m   n   o   p   q
            0000021
            "));
}

#[test]
fn test_skip_bytes_error(){
    let input = "12345";
    let result = new_ucmd!().arg("--skip-bytes=10").run_piped_stdin(input.as_bytes());

    assert!(!result.success);
}

#[test]
fn test_read_bytes(){
    let input = "abcdefghijklmnopqrstuvwxyz\n12345678";
    let result = new_ucmd!().arg("--endian=little").arg("--read-bytes=27").run_piped_stdin(input.as_bytes());

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, unindent(ALPHA_OUT));
}

#[test]
fn test_ascii_dump(){
    let input: [u8; 22] = [
        0x00, 0x01, 0x0a, 0x0d, 0x10, 0x1f, 0x20, 0x61, 0x62, 0x63, 0x7d,
        0x7e, 0x7f, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0, 0xff];
    let result = new_ucmd!().arg("-tx1zacz").run_piped_stdin(&input[..]);

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, unindent(r"
            0000000  00  01  0a  0d  10  1f  20  61  62  63  7d  7e  7f  80  90  a0  >...... abc}~....<
                    nul soh  nl  cr dle  us  sp   a   b   c   }   ~ del nul dle  sp
                     \0 001  \n  \r 020 037       a   b   c   }   ~ 177  **  **  **  >...... abc}~....<
            0000020  b0  c0  d0  e0  f0  ff                                          >......<
                      0   @   P   `   p del
                     ** 300 320 340 360 377                                          >......<
            0000026
            "));
}

#[test]
fn test_filename_parsing(){
    // files "a" and "x" both exists, but are no filenames in the commandline below
    // "-f" must be treated as a filename, it contains the text: minus lowercase f
    // so "-f" should not be interpreted as a formatting option.
    let result = new_ucmd!().arg("--format").arg("a").arg("-A").arg("x").arg("--").arg("-f").run();

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, unindent("
            000000   m   i   n   u   s  sp   l   o   w   e   r   c   a   s   e  sp
            000010   f  nl
            000012
            "));
}

#[test]
fn test_stdin_offset(){
    let input = "abcdefghijklmnopq";
    let result = new_ucmd!().arg("-c").arg("+5").run_piped_stdin(input.as_bytes());

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, unindent("
            0000005   f   g   h   i   j   k   l   m   n   o   p   q
            0000021
            "));
}

#[test]
fn test_file_offset(){
    let result = new_ucmd!().arg("-c").arg("--").arg("-f").arg("10").run();

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, unindent(r"
            0000010   w   e   r   c   a   s   e       f  \n
            0000022
            "));
}

#[test]
fn test_traditional(){
    // note gnu od does not align both lines
    let input = "abcdefghijklmnopq";
    let result = new_ucmd!().arg("--traditional").arg("-a").arg("-c").arg("-").arg("10").arg("0").run_piped_stdin(input.as_bytes());

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, unindent(r"
            0000010 (0000000)   i   j   k   l   m   n   o   p   q
                                i   j   k   l   m   n   o   p   q
            0000021 (0000011)
            "));
}

#[test]
fn test_traditional_with_skip_bytes_override(){
    // --skip-bytes is ignored in this case
    let input = "abcdefghijklmnop";
    let result = new_ucmd!().arg("--traditional").arg("--skip-bytes=10").arg("-c").arg("0").run_piped_stdin(input.as_bytes());

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, unindent(r"
            0000000   a   b   c   d   e   f   g   h   i   j   k   l   m   n   o   p
            0000020
            "));
}

#[test]
fn test_traditional_with_skip_bytes_non_override(){
    // no offset specified in the traditional way, so --skip-bytes is used
    let input = "abcdefghijklmnop";
    let result = new_ucmd!().arg("--traditional").arg("--skip-bytes=10").arg("-c").run_piped_stdin(input.as_bytes());

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, unindent(r"
            0000012   k   l   m   n   o   p
            0000020
            "));
}

#[test]
fn test_traditional_error(){
    // file "0" exists - don't fail on that, but --traditional only accepts a single input
    let result = new_ucmd!().arg("--traditional").arg("0").arg("0").arg("0").arg("0").run();

    assert!(!result.success);
}

#[test]
fn test_traditional_only_label(){
    let input = "abcdefghijklmnopqrstuvwxyz";
    let result = new_ucmd!().arg("-An").arg("--traditional").arg("-a").arg("-c").arg("-").arg("10").arg("0x10").run_piped_stdin(input.as_bytes());

    assert_empty_stderr!(result);
    assert!(result.success);
    assert_eq!(result.stdout, unindent(r"
            (0000020)   i   j   k   l   m   n   o   p   q   r   s   t   u   v   w   x
                        i   j   k   l   m   n   o   p   q   r   s   t   u   v   w   x
            (0000040)   y   z
                        y   z
            (0000042)
            "));
}
