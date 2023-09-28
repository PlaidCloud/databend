// Copyright 2021 Datafuse Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::io::Cursor;
use std::io::Read;

use common_expression::TableSchemaRef;
use common_io::format_diagnostic::verbose_char;
use common_storage::FileParseError;

pub fn truncate_column_data(s: String) -> String {
    if s.len() > 100 {
        s.chars().take(100).collect::<String>()
    } else {
        s
    }
}

pub fn get_decode_error_by_pos(
    column_index: usize,
    schema: &TableSchemaRef,
    decode_error: &str,
    column_data: &[u8],
) -> FileParseError {
    let field = &schema.fields()[column_index];
    let column_data = String::from_utf8_lossy(column_data).to_string();
    FileParseError::ColumnDecodeError {
        column_index,
        decode_error: decode_error.to_string(),
        column_name: field.name().to_string(),
        column_type: field.data_type().to_string(),
        column_data: truncate_column_data(column_data),
    }
}

pub(crate) fn check_column_end(
    reader: &mut Cursor<&[u8]>,
    schema: &TableSchemaRef,
    column_index: usize,
) -> std::result::Result<(), FileParseError> {
    let mut next = [0u8; 1];
    // read from Cursor never returns Err
    let readn = reader.read(&mut next[..]).unwrap();

    if readn > 0 {
        let size_remained = reader.remaining_slice().len() + 1;
        let field = &schema.fields()[column_index];
        let error_pos = reader.position() as usize - 1;
        reader.set_position(0);
        Err(FileParseError::ColumnDataNotDrained {
            column_index: column_index as u32,
            size_remained: size_remained as u32,
            error_pos: error_pos as u32,
            column_name: field.name().to_string(),
            column_type: field.data_type().to_string(),
            next_char: verbose_char(next[0]),
            column_data: truncate_column_data(
                String::from_utf8_lossy(reader.remaining_slice()).to_string(),
            ),
        })
    } else {
        Ok(())
    }
}