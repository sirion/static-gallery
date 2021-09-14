#![cfg(test)]

use crate::gallery::CollectionInput;
use crate::gallery::Gallery;
use lazy_static::lazy_static;
use std::path::PathBuf;
use std::str::FromStr;

// fn dir_tpl() -> PathBuf { PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("data").join("templates").join("hauer") }
fn dir_td() -> PathBuf {
	PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
		.join("data")
		.join("test")
}
// fn dir_out() -> PathBuf { dir_td().join("output") }
fn dir_in() -> PathBuf {
	dir_td().join("input01")
}
fn dir_in2() -> PathBuf {
	dir_td().join("input02")
}
fn dir_in3() -> PathBuf {
	dir_td().join("input03")
}
fn dir_bg() -> PathBuf {
	dir_td().join("backgrounds01")
}
fn dir_bg2() -> PathBuf {
	dir_td().join("backgrounds02")
}
fn dir_bg3() -> PathBuf {
	dir_td().join("backgrounds03")
}
fn dir_none() -> PathBuf {
	PathBuf::from("-")
}

fn create_input(input: PathBuf, backgrounds: PathBuf, title: &str) -> CollectionInput {
	let input = input.to_string_lossy();
	let backgrounds = backgrounds.to_string_lossy();
	let col_str = vec![input.as_ref(), backgrounds.as_ref(), title];
	CollectionInput::from_str(col_str.join(";").as_str()).unwrap()
}

struct FileCounts {
	in1: usize,
	in2: usize,
	in3: usize,
	bg1: usize,
	bg2: usize,
	bg3: usize,
}

lazy_static! {
	static ref FC: FileCounts = FileCounts {
		in1: std::fs::read_dir(dir_in()).unwrap().collect::<Vec<_>>().len(),
		in2: std::fs::read_dir(dir_in2()).unwrap().collect::<Vec<_>>().len(),
		in3: std::fs::read_dir(dir_in3()).unwrap().collect::<Vec<_>>().len(),
		bg1: std::fs::read_dir(dir_bg()).unwrap().collect::<Vec<_>>().len(),
		bg2: std::fs::read_dir(dir_bg2()).unwrap().collect::<Vec<_>>().len(),
		bg3: std::fs::read_dir(dir_bg3()).unwrap().collect::<Vec<_>>().len(),
	};
}

// TESTS: Create

//   Create single collection with one background source
//       -o out_dir/ -p template_dir/ -c "input_dir1/;bg_dir1/;Col 1"
#[test]
fn test_create_collection_one_background() {
	let mut gallery = Gallery::new();

	let input_col1 = create_input(dir_in(), dir_bg(), "Col 1");

	gallery.fill(vec![input_col1], false).unwrap();

	assert_eq!(gallery.collection_keys.len(), 1);
	let col = &gallery.collections[&gallery.collection_keys[0]];
	assert_eq!(col.title, "Col 1");
	assert_eq!(col.pictures.len(), FC.in1);
	assert_eq!(col.backgrounds.len(), FC.bg1);
}

//   Create multiple collections with one background source
//       -o out_dir/ -p template_dir/ -c "input_dir1/;bg_dir1/;Col 1" -c "input_dir2/;-;Col 2"
#[test]
fn test_create_two_collections_one_background() {
	let mut gallery = Gallery::new();

	let input_col1 = create_input(dir_in(), dir_bg(), "Col 1");
	let input_col2 = create_input(dir_in2(), dir_none(), "Col 2");

	gallery.fill(vec![input_col1, input_col2], false).unwrap();

	assert_eq!(gallery.collection_keys.len(), 2);

	let col = &gallery.collections[&gallery.collection_keys[0]];
	assert_eq!(col.title, "Col 1");
	assert_eq!(col.pictures.len(), FC.in1);
	assert_eq!(col.backgrounds.len(), FC.bg1);

	let col = &gallery.collections[&gallery.collection_keys[1]];
	assert_eq!(col.title, "Col 2");
	assert_eq!(col.pictures.len(), FC.in2);
	assert_eq!(col.backgrounds.len(), 0);
}

//   Create multiple collections with multiple background sources
//       -o out_dir/ -p template_dir/ -c "input_dir1/;bg_dir1/;Col 1" -c "input_dir2/;bg_dir2/;Col 2" -c "input_dir3/;-;Col 3"
#[test]
fn test_create_three_collections_two_backgrounds() {
	let mut gallery = Gallery::new();

	let input_col1 = create_input(dir_in(), dir_bg(), "Col 1");
	let input_col2 = create_input(dir_in2(), dir_bg2(), "Col 2");
	let input_col3 = create_input(dir_in3(), dir_none(), "Col 3");

	gallery
		.fill(vec![input_col1, input_col2, input_col3], false)
		.unwrap();

	assert_eq!(gallery.collection_keys.len(), 3);

	let col = &gallery.collections[&gallery.collection_keys[0]];
	assert_eq!(col.title, "Col 1");
	assert_eq!(col.pictures.len(), FC.in1);
	assert_eq!(col.backgrounds.len(), FC.bg1);

	let col = &gallery.collections[&gallery.collection_keys[1]];
	assert_eq!(col.title, "Col 2");
	assert_eq!(col.pictures.len(), FC.in2);
	assert_eq!(col.backgrounds.len(), FC.bg2);

	let col = &gallery.collections[&gallery.collection_keys[2]];
	assert_eq!(col.title, "Col 3");
	assert_eq!(col.pictures.len(), FC.in3);
	assert_eq!(col.backgrounds.len(), 0);
}

//   Create multiple collections with one background source each
//       -o out_dir/ -p template_dir/ -c "input_dir1/;bg_dir1/;Col 1" -c "input_dir2/;bg_dir2/;Col 2" -c "input_dir3/;bg_dir3/;Col 3"
#[test]
fn test_create_three_collections_three_backgrounds() {
	let mut gallery = Gallery::new();

	let input_col1 = create_input(dir_in(), dir_bg(), "Col 1");
	let input_col2 = create_input(dir_in2(), dir_bg2(), "Col 2");
	let input_col3 = create_input(dir_in3(), dir_bg3(), "Col 3");

	gallery
		.fill(vec![input_col1, input_col2, input_col3], false)
		.unwrap();

	assert_eq!(gallery.collection_keys.len(), 3);

	let col = &gallery.collections[&gallery.collection_keys[0]];
	assert_eq!(col.title, "Col 1");
	assert_eq!(col.pictures.len(), FC.in1);
	assert_eq!(col.backgrounds.len(), FC.bg1);

	let col = &gallery.collections[&gallery.collection_keys[1]];
	assert_eq!(col.title, "Col 2");
	assert_eq!(col.pictures.len(), FC.in2);
	assert_eq!(col.backgrounds.len(), FC.bg2);

	let col = &gallery.collections[&gallery.collection_keys[2]];
	assert_eq!(col.title, "Col 3");
	assert_eq!(col.pictures.len(), FC.in3);
	assert_eq!(col.backgrounds.len(), FC.bg3);
}

//   Create multiple collections without background sources
//       -o out_dir/ -p template_dir/ -c "input_dir1/;-;Col 1" -c "input_dir2/;-;Col 2" -c "input_dir3/;-;Col 3"
#[test]
fn test_create_three_collections_no_backgrounds() {
	let mut gallery = Gallery::new();

	let input_col1 = create_input(dir_in(), dir_none(), "Col 1");
	let input_col2 = create_input(dir_in2(), dir_none(), "Col 2");
	let input_col3 = create_input(dir_in3(), dir_none(), "Col 3");

	gallery
		.fill(vec![input_col1, input_col2, input_col3], false)
		.unwrap();

	assert_eq!(gallery.collection_keys.len(), 3);

	let col = &gallery.collections[&gallery.collection_keys[0]];
	assert_eq!(col.title, "Col 1");
	assert_eq!(col.pictures.len(), FC.in1);
	assert_eq!(col.backgrounds.len(), 0);

	let col = &gallery.collections[&gallery.collection_keys[1]];
	assert_eq!(col.title, "Col 2");
	assert_eq!(col.pictures.len(), FC.in2);
	assert_eq!(col.backgrounds.len(), 0);

	let col = &gallery.collections[&gallery.collection_keys[2]];
	assert_eq!(col.title, "Col 3");
	assert_eq!(col.pictures.len(), FC.in3);
	assert_eq!(col.backgrounds.len(), 0);
}

// TESTS: Update

//   Add new collection with new backgrounds
//       -o out_dir/ -p template_dir/ -c "input_dir1/;bg_dir1/;Col 1"
//       -u -o out_dir/ -c "input_dir2/;bg_dir2/;Col Added"
#[test]
fn test_update_one_collection_one_background() {
	let mut gallery = Gallery::new();

	let input_col1 = create_input(dir_in(), dir_bg(), "Col 1");
	gallery.fill(vec![input_col1], false).unwrap();

	assert_eq!(gallery.collection_keys.len(), 1);

	let input_col2 = create_input(dir_in2(), dir_bg2(), "Col Added");
	gallery.fill(vec![input_col2], false).unwrap();

	assert_eq!(gallery.collection_keys.len(), 2);

	//panic!("{:#?}", gallery);

	let col = &gallery.collections[&gallery.collection_keys[0]];
	assert_eq!(col.title, "Col 1");
	assert_eq!(col.pictures.len(), FC.in1);
	assert_eq!(col.backgrounds.len(), FC.bg1);

	let col = &gallery.collections[&gallery.collection_keys[1]];
	assert_eq!(col.title, "Col Added");
	assert_eq!(col.pictures.len(), FC.in2);
	assert_eq!(col.backgrounds.len(), FC.bg2);
}

//   Add new collection without backgrounds
//       -o out_dir/ -p template_dir/ -c "input_dir1/;bg_dir1/;Col 1"
//       -u -o out_dir/ -c "input_dir2/;-;Col Added"
#[test]
fn test_update_one_collection_no_background() {
	let mut gallery = Gallery::new();

	let input_col1 = create_input(dir_in(), dir_bg(), "Col 1");
	gallery.fill(vec![input_col1], false).unwrap();

	assert_eq!(gallery.collection_keys.len(), 1);

	let input_col2 = create_input(dir_in2(), dir_none(), "Col Added");
	gallery.fill(vec![input_col2], false).unwrap();

	assert_eq!(gallery.collection_keys.len(), 2);

	let col = &gallery.collections[&gallery.collection_keys[0]];
	assert_eq!(col.title, "Col 1");
	assert_eq!(col.pictures.len(), FC.in1);
	assert_eq!(col.backgrounds.len(), FC.bg1);

	let col = &gallery.collections[&gallery.collection_keys[1]];
	assert_eq!(col.title, "Col Added");
	assert_eq!(col.pictures.len(), FC.in2);
	assert_eq!(col.backgrounds.len(), 0);
}

//   Update collection without backgrounds
//       -o out_dir/ -p template_dir/ -c "input_dir1/;bg_dir1/;Col 1"
//       -u -o out_dir/ -c "input_dir2/;-;Col 1"
#[test]
fn test_update_existing_collection_no_background() {
	let mut gallery = Gallery::new();

	let input_col1 = create_input(dir_in(), dir_bg(), "Col 1");

	gallery.fill(vec![input_col1], false).unwrap();

	assert_eq!(gallery.collection_keys.len(), 1);

	let col = &gallery.collections[&gallery.collection_keys[0]];
	assert_eq!(col.title, "Col 1");
	assert_eq!(col.pictures.len(), FC.in1);
	assert_eq!(col.backgrounds.len(), FC.bg1);

	let input_col2 = create_input(dir_in2(), dir_none(), "Col 1");

	gallery.fill(vec![input_col2], false).unwrap();

	assert_eq!(gallery.collection_keys.len(), 1);

	let col = &gallery.collections[&gallery.collection_keys[0]];
	assert_eq!(col.title, "Col 1");
	assert_eq!(col.pictures.len(), FC.in1 + FC.in2);
	assert_eq!(col.backgrounds.len(), FC.bg1);
}

//   Update collection with new pictures and backgrounds
//       -o out_dir/ -p template_dir/ -c "input_dir1/;bg_dir1/;Col 1"
//       -u -o out_dir/ -c "input_dir2/;bg_dir2/;Col 1"
#[test]
fn test_update_existing_collection_new_pictures_and_backgrounds() {
	let mut gallery = Gallery::new();

	let input_col1 = create_input(dir_in(), dir_bg(), "Col 1");
	gallery.fill(vec![input_col1], false).unwrap();

	assert_eq!(gallery.collection_keys.len(), 1);

	let col = &gallery.collections[&gallery.collection_keys[0]];
	assert_eq!(col.title, "Col 1");
	assert_eq!(col.pictures.len(), FC.in1);
	assert_eq!(col.backgrounds.len(), FC.bg1);

	let input_col2 = create_input(dir_in2(), dir_bg2(), "Col 1");

	gallery.fill(vec![input_col2], false).unwrap();

	assert_eq!(gallery.collection_keys.len(), 1);

	let col = &gallery.collections[&gallery.collection_keys[0]];
	assert_eq!(col.title, "Col 1");
	assert_eq!(col.pictures.len(), FC.in1 + FC.in2);
	assert_eq!(col.backgrounds.len(), FC.bg1 + FC.bg2);
}

//   Update collection with only new backgrounds
//       -o out_dir/ -p template_dir/ -c "input_dir1/;bg_dir1/;Col 1"
//       -u -o out_dir/ -c "-;bg_dir2/;Col 1"
#[test]
fn test_update_existing_collection_new_backgrounds() {
	let mut gallery = Gallery::new();

	let input_col1 = create_input(dir_in(), dir_bg(), "Col 1");

	gallery.fill(vec![input_col1], false).unwrap();

	assert_eq!(gallery.collection_keys.len(), 1);

	let col = &gallery.collections[&gallery.collection_keys[0]];
	assert_eq!(col.title, "Col 1");
	assert_eq!(col.pictures.len(), FC.in1);
	assert_eq!(col.backgrounds.len(), FC.bg1);

	let input_col2 = create_input(dir_none(), dir_bg2(), "Col 1");

	gallery.fill(vec![input_col2], false).unwrap();

	assert_eq!(gallery.collection_keys.len(), 1);

	let col = &gallery.collections[&gallery.collection_keys[0]];
	assert_eq!(col.title, "Col 1");
	assert_eq!(col.pictures.len(), FC.in1);
	assert_eq!(col.backgrounds.len(), FC.bg1 + FC.bg2);
}

//   Update collection with the same pictures and backgounds
//       -o out_dir/ -p template_dir/ -c "input_dir1/;bg_dir1/;Col 1"
//       -u -o out_dir/ -c "input_dir1/;bg_dir1/;Col 1"
#[test]
fn test_update_existing_collection_same_pictures_and_backgrounds() {
	let mut gallery = Gallery::new();

	let input_col1 = create_input(dir_in(), dir_bg(), "Col 1");

	gallery.fill(vec![input_col1], false).unwrap();

	assert_eq!(gallery.collection_keys.len(), 1);

	let col = &gallery.collections[&gallery.collection_keys[0]];
	assert_eq!(col.title, "Col 1");
	assert_eq!(col.pictures.len(), FC.in1);
	assert_eq!(col.backgrounds.len(), FC.bg1);

	let input_col2 = create_input(dir_in(), dir_bg(), "Col 1");

	gallery.fill(vec![input_col2], false).unwrap();

	assert_eq!(gallery.collection_keys.len(), 1);

	let col = &gallery.collections[&gallery.collection_keys[0]];
	assert_eq!(col.title, "Col 1");
	assert_eq!(col.pictures.len(), FC.in1 * 2);
	assert_eq!(col.backgrounds.len(), FC.bg1 * 2);

	gallery.remove_duplicates();

	// println!("{:#?}", gallery);

	// Count images that need an update
	let mut num_updates = 0;
	for (_, c) in &gallery.collections {
		for p in &c.pictures {
			if p.image.update {
				num_updates += 1;
			}
		}
		for b in &c.backgrounds {
			if b.update {
				num_updates += 1;
			}
		}
	}

	assert_eq!(num_updates, FC.in1 + FC.bg1, "Updatablae files should only be half of the added files ({}) it is {}", FC.in1 + FC.bg1, num_updates);
}
