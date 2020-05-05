// Based on:
//   https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT
//   .ZIP File Format Specification - Version: 6.3.6

#![feature(seek_convenience)]

use byteorder::{LittleEndian as LE, ReadBytesExt, WriteBytesExt};
use clap::{App, Arg};
use encoding::all::WINDOWS_31J;
use encoding::{DecoderTrap, Encoding};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, Write};

fn main() -> Result<(), std::io::Error> {
    let matches = App::new("fix-japanese-zip")
        .author("Nagy Tibor <xnagytibor@gmail.com>")
        .about("Converts Shift JIS encoded ZIP files to UTF-8 without recompression")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .help("Sets the input ZIP file")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .help("Sets the output ZIP file")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let mut input = File::open(matches.value_of("input").unwrap())?;
    let mut output = File::create(matches.value_of("output").unwrap())?;

    let mut output_file_header_offsets = HashMap::new();
    let mut output_central_directory_offset = 0;

    while input.stream_position()? < input.stream_len()? {
        let input_record_offset = input.stream_position()? as u32;
        let output_record_offset = output.stream_position()? as u32;

        let mut record_signature: [u8; 4] = [0, 0, 0, 0];
        input.read_exact(&mut record_signature)?;

        match &record_signature {
            // Local file header
            &[b'P', b'K', 0x03, 0x04] => {
                // Read the fixed-length fields
                let src_version_needed_to_extract = input.read_u16::<LE>()?;
                let src_general_purpose_bit_flag = input.read_u16::<LE>()?;
                let src_compression_method = input.read_u16::<LE>()?;
                let src_last_mod_file_time = input.read_u16::<LE>()?;
                let src_last_mod_file_date = input.read_u16::<LE>()?;
                let src_crc32 = input.read_u32::<LE>()?;
                let src_compressed_size = input.read_u32::<LE>()?;
                let src_uncompressed_size = input.read_u32::<LE>()?;
                let src_file_name_length = input.read_u16::<LE>()?;
                let src_extra_field_length = input.read_u16::<LE>()?;

                // Read the variable-length fields
                let mut src_file_name = vec![0; src_file_name_length as usize];
                let mut src_extra_field = vec![0; src_extra_field_length as usize];
                input.read_exact(&mut src_file_name)?;
                input.read_exact(&mut src_extra_field)?;

                // Convert file name from Shift JIS to UTF-8
                let dst_file_name = WINDOWS_31J
                    .decode(&src_file_name, DecoderTrap::Strict)
                    .unwrap();
                let dst_file_name_length = dst_file_name.len() as u16;

                // Emit the converted local file header
                output.write_all(&[b'P', b'K', 0x03, 0x04])?;
                output.write_u16::<LE>(src_version_needed_to_extract)?;
                output.write_u16::<LE>(src_general_purpose_bit_flag)?;
                output.write_u16::<LE>(src_compression_method)?;
                output.write_u16::<LE>(src_last_mod_file_time)?;
                output.write_u16::<LE>(src_last_mod_file_date)?;
                output.write_u32::<LE>(src_crc32)?;
                output.write_u32::<LE>(src_compressed_size)?;
                output.write_u32::<LE>(src_uncompressed_size)?;
                output.write_u16::<LE>(dst_file_name_length)?;
                output.write_u16::<LE>(src_extra_field_length)?;
                output.write_all(dst_file_name.as_bytes())?;
                output.write_all(&src_extra_field)?;

                // Copy compressed data (TODO: Copy by chunks)
                let mut src_data = vec![0; src_compressed_size as usize];
                input.read_exact(&mut src_data)?;
                output.write_all(&src_data)?;

                // Save record offset for the central directory later
                output_file_header_offsets.insert(dst_file_name, output_record_offset);
            }

            // File header
            &[b'P', b'K', 0x01, 0x02] => {
                if output_central_directory_offset == 0 {
                    output_central_directory_offset = output_record_offset;
                }

                // Read the fixed-length fields
                let src_version_made_by = input.read_u16::<LE>()?;
                let src_version_needed_to_extract = input.read_u16::<LE>()?;
                let src_general_purpose_bit_flag = input.read_u16::<LE>()?;
                let src_compression_method = input.read_u16::<LE>()?;
                let src_last_mod_file_time = input.read_u16::<LE>()?;
                let src_last_mod_file_date = input.read_u16::<LE>()?;
                let src_crc32 = input.read_u32::<LE>()?;
                let src_compressed_size = input.read_u32::<LE>()?;
                let src_uncompressed_size = input.read_u32::<LE>()?;
                let src_file_name_length = input.read_u16::<LE>()?;
                let src_extra_field_length = input.read_u16::<LE>()?;
                let src_file_comment_length = input.read_u16::<LE>()?;
                let src_disk_number_start = input.read_u16::<LE>()?;
                let src_internal_file_attributes = input.read_u16::<LE>()?;
                let src_external_file_attributes = input.read_u32::<LE>()?;
                let _src_relative_offset_of_local_header = input.read_u32::<LE>()?;

                // Read the variable-length fields
                let mut src_file_name = vec![0; src_file_name_length as usize];
                let mut src_extra_field = vec![0; src_extra_field_length as usize];
                let mut src_file_comment = vec![0; src_file_comment_length as usize];
                input.read_exact(&mut src_file_name)?;
                input.read_exact(&mut src_extra_field)?;
                input.read_exact(&mut src_file_comment)?; // TODO: Does this need conversion?

                // Convert file name from Shift JIS to UTF-8
                let dst_file_name = WINDOWS_31J
                    .decode(&src_file_name, DecoderTrap::Strict)
                    .unwrap();
                let dst_file_name_length = dst_file_name.len() as u16;

                // Emit the converted file header
                output.write_all(&[b'P', b'K', 0x01, 0x02])?;
                output.write_u16::<LE>(src_version_made_by)?;
                output.write_u16::<LE>(src_version_needed_to_extract)?;
                output.write_u16::<LE>(src_general_purpose_bit_flag)?;
                output.write_u16::<LE>(src_compression_method)?;
                output.write_u16::<LE>(src_last_mod_file_time)?;
                output.write_u16::<LE>(src_last_mod_file_date)?;
                output.write_u32::<LE>(src_crc32)?;
                output.write_u32::<LE>(src_compressed_size)?;
                output.write_u32::<LE>(src_uncompressed_size)?;
                output.write_u16::<LE>(dst_file_name_length)?;
                output.write_u16::<LE>(src_extra_field_length)?;
                output.write_u16::<LE>(src_file_comment_length)?;
                output.write_u16::<LE>(src_disk_number_start)?;
                output.write_u16::<LE>(src_internal_file_attributes)?;
                output.write_u32::<LE>(src_external_file_attributes)?;
                output.write_u32::<LE>(output_file_header_offsets[&dst_file_name])?;
                output.write_all(dst_file_name.as_bytes())?;
                output.write_all(&src_extra_field)?;
                output.write_all(&src_file_comment)?;
            }

            // End of central directory record
            &[b'P', b'K', 0x05, 0x06] => {
                // Read the fixed-length fields
                let src_disk_number = input.read_u16::<LE>()?;
                let src_disk_with_cdir = input.read_u16::<LE>()?;
                let src_cdir_entries_this_disk = input.read_u16::<LE>()?;
                let src_cdir_entries = input.read_u16::<LE>()?;
                let _src_size_of_the_cdir = input.read_u32::<LE>()?;
                let _src_offset_of_start_of_cdir = input.read_u32::<LE>()?;
                let src_file_comment_length = input.read_u16::<LE>()?;

                // Read the variable-length fields
                let mut src_file_comment = vec![0; src_file_comment_length as usize];
                input.read_exact(&mut src_file_comment)?; // TODO: Does this need conversion?

                // TODO: Should the size of this end record included in the central directory size?
                let dst_size_of_the_cdir = output_record_offset - output_central_directory_offset;

                // Emit the converted end record
                output.write_all(&[b'P', b'K', 0x05, 0x06])?;
                output.write_u16::<LE>(src_disk_number)?;
                output.write_u16::<LE>(src_disk_with_cdir)?;
                output.write_u16::<LE>(src_cdir_entries_this_disk)?;
                output.write_u16::<LE>(src_cdir_entries)?;
                output.write_u32::<LE>(dst_size_of_the_cdir)?;
                output.write_u32::<LE>(output_central_directory_offset)?;
                output.write_u16::<LE>(src_file_comment_length)?;
                output.write_all(&src_file_comment)?;
            }

            &[b'P', b'K', 0x06, 0x08] => unimplemented!("Archive extra data record"),
            &[b'P', b'K', 0x05, 0x05] => unimplemented!("Digital signature"),
            &[b'P', b'K', 0x06, 0x06] => unimplemented!("Zip64 end of central directory record"),
            &[b'P', b'K', 0x06, 0x07] => unimplemented!("Zip64 end of central directory locator"),
            &[b'P', b'K', 0x07, 0x08] => unimplemented!("Special spanning signature"),
            &[b'P', b'K', 0x30, 0x30] => unimplemented!("Special spanning marker"),

            record_signature => panic!(
                "Unknown record {:?} at offset 0x{:08X}",
                record_signature, input_record_offset
            ),
        }
    }

    Ok(())
}
