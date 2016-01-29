use std::marker::PhantomData;
use std::iter::Iterator;
use std::ptr::null;
use std::str::from_utf8_unchecked;
use std::slice::from_raw_parts;
use libc::{c_int, c_uint, c_ulong, c_void, c_uchar};

use onig_sys;

use super::Regex;

impl Regex {
    /// Returns the number of named groups into regex.
    pub fn names_len(&self) -> usize {
        unsafe { onig_sys::onig_number_of_names(self.raw) as usize }
    }

    /// Returns the iterator over named groups as a tuple with the group name
    /// and group indexes.
    pub fn names<'r>(&'r self) -> Names<'r> {
        Names {
            table: unsafe { (*self.raw).name_table as *const StTable },
            bin_idx: -1,
            entry_ptr: null(),
            _phantom: PhantomData
        }
    }
}

#[repr(C)]
#[derive(Debug)]
struct NameEntry {
    name: *const c_uchar,
    name_len: c_int,
    back_num: c_int,
    back_alloc: c_int,
    back_ref1: c_int,
    back_refs: *const c_int
}

#[repr(C)]
#[derive(Debug)]
struct StTableEntry {
    hash: c_uint,
    key: c_ulong,
    record: c_ulong,
    next: *const StTableEntry
}

#[repr(C)]
#[derive(Debug)]
struct StTable {
    type_: *const c_void,
    num_bins: c_int,
    num_entries: c_int,
    bins: *const *const StTableEntry
}

/// Names is an iterator over named groups as a tuple with the group name
/// and group indexes.
///
/// `'r` is the lifetime of the Regex object.
#[derive(Debug)]
pub struct Names<'r> {
    table: *const StTable,
    bin_idx: c_int,
    entry_ptr: *const StTableEntry,
    _phantom: PhantomData<&'r Regex>
}

impl<'r> Iterator for Names<'r> {
    type Item = (&'r str, &'r [i32]);

    fn next(&mut self) -> Option<(&'r str, &'r [i32])> {
        unsafe {
            while self.entry_ptr.is_null() {
                if self.table.is_null() || self.bin_idx + 1 >= (*self.table).num_bins {
                    return None
                }
                self.bin_idx += 1;
                self.entry_ptr = *(*self.table).bins.offset(self.bin_idx as isize)
            }
            let entry = (*self.entry_ptr).record as *const NameEntry;
            let name = from_utf8_unchecked(
                from_raw_parts((*entry).name, (*entry).name_len as usize)
            );
            let groups = if (*entry).back_num > 1 {
                from_raw_parts((*entry).back_refs, (*entry).back_num as usize)
            } else {
                from_raw_parts(&(*entry).back_ref1, 1)
            };
            self.entry_ptr = (*self.entry_ptr).next;
            Some((name, groups))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_regex_names_len() {
        let regex = Regex::new("(he)(l+)(o)").unwrap();
        assert_eq!(regex.names_len(), 0);
        let regex = Regex::new("(?<foo>he)(?<bar>l+)(?<bar>o)").unwrap();
        assert_eq!(regex.names_len(), 2);
    }

    #[test]
    fn test_regex_names() {
        let regex = Regex::new("(he)(l+)(o)").unwrap();
        let names = regex.names().collect::<Vec<_>>();
        assert_eq!(names, vec![]);
        let regex = Regex::new("(?<foo>he)(?<bar>l+)(?<bar>o)").unwrap();
        let names = regex.names().collect::<Vec<_>>();
        assert_eq!(names,
                   [("foo", &[1] as &[i32]), ("bar", &[2, 3] as &[i32])]);
    }
}


