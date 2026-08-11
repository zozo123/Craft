//! Exact parity for the item/block tables and material predicates.
//! These are integer data, so equality is exact on every platform.

mod common;

use craft_core::item;

#[test]
fn items_blocks_plants_predicates_match_c() {
    let contents = common::read_golden("item.tsv");

    // item_count from the leading comment line.
    let count_line = contents
        .lines()
        .find(|l| l.starts_with("# item_count"))
        .expect("missing item_count header");
    let count: usize = count_line.rsplit('\t').next().unwrap().parse().unwrap();
    assert_eq!(count, item::ITEM_COUNT, "item_count mismatch");

    let mut checked_items = 0;
    let mut checked_blocks = 0;
    let mut checked_plants = 0;
    let mut checked_preds = 0;
    let mut checked_negpreds = 0;

    for r in common::rows(&contents) {
        match r[0] {
            "item" => {
                let i: usize = r[1].parse().unwrap();
                let v: i32 = r[2].parse().unwrap();
                assert_eq!(item::ITEMS[i], v, "ITEMS[{i}]");
                checked_items += 1;
            }
            "block" => {
                let w: usize = r[1].parse().unwrap();
                let expect: [i32; 6] = std::array::from_fn(|k| r[2 + k].parse().unwrap());
                assert_eq!(item::BLOCKS[w], expect, "BLOCKS[{w}]");
                checked_blocks += 1;
            }
            "plant" => {
                let w: usize = r[1].parse().unwrap();
                let v: i32 = r[2].parse().unwrap();
                assert_eq!(item::PLANTS[w], v, "PLANTS[{w}]");
                checked_plants += 1;
            }
            "pred" | "negpred" => {
                let w: i32 = r[1].parse().unwrap();
                let (p, o, t, d) = (r[2] == "1", r[3] == "1", r[4] == "1", r[5] == "1");
                assert_eq!(item::is_plant(w), p, "is_plant({w})");
                assert_eq!(item::is_obstacle(w), o, "is_obstacle({w})");
                assert_eq!(item::is_transparent(w), t, "is_transparent({w})");
                assert_eq!(item::is_destructable(w), d, "is_destructable({w})");
                if r[0] == "pred" {
                    checked_preds += 1;
                } else {
                    checked_negpreds += 1;
                }
            }
            other => panic!("unexpected row kind: {other}"),
        }
    }

    assert_eq!(checked_items, item::ITEM_COUNT);
    assert_eq!(checked_blocks, 256);
    assert_eq!(checked_plants, 256);
    assert_eq!(checked_preds, 256);
    assert_eq!(checked_negpreds, 64);
}
