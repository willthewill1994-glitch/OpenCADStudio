//! Named style inheritance must survive nested block batching and prototype reuse.
use document::entities::{Insert, Line};
use document::objects::{ObjectType, PlaceHolder};
use document::tables::BlockRecord;
use document::types::Vector3;
use document::{Document, EntityType, Handle};
use OpenCADStudio::scene::cache::block_cache::{expand_insert, BlockCache};
use OpenCADStudio::scene::model::instance_model::expand_stubs;
use OpenCADStudio::scene::view::render::InheritStyle;
use OpenCADStudio::scene::{BlockScalePolicy, Scene};

fn pen(doc: &mut Document, name: &str) -> Handle {
    let mut p = PlaceHolder::new();
    p.handle = doc.allocate_handle();
    p.owner = doc.header.acad_plotstylename_dict_handle;
    let handle = p.handle;
    doc.objects.insert(handle, ObjectType::PlaceHolder(p));
    let dictionary = doc.header.acad_plotstylename_dict_handle;
    match doc
        .objects
        .get_mut(&dictionary)
        .unwrap()
    {
        ObjectType::DictionaryWithDefault(d) => d.entries.push((name.into(), handle)),
        _ => panic!("plot dictionary"),
    }
    handle
}

#[test]
fn nested_children_keep_named_pens_across_batches_and_repeated_inserts() {
    let mut scene = Scene::new();
    let doc = &mut scene.document;
    doc.header.plotstyle_mode = false;
    let blue = pen(doc, "Blue");
    let green = pen(doc, "Green");
    doc.layers.get_mut("0").unwrap().plotstyle_handle = green;
    let mut leaf = BlockRecord::new("STB_LEAF");
    leaf.handle = doc.allocate_handle();
    let leaf_handle = leaf.handle;
    doc.block_records.add(leaf).unwrap();
    for flags in [0, 1, 3] {
        let mut line = Line::new();
        line.start = Vector3::new(0.0, flags as f64, 0.0);
        line.end = Vector3::new(10.0, flags as f64, 0.0);
        line.common.owner_handle = leaf_handle;
        line.common.plotstyle_flags = flags;
        line.common.plotstyle_handle = (flags == 3).then_some(blue);
        doc.add_entity(EntityType::Line(line)).unwrap();
    }
    let mut outer = BlockRecord::new("STB_OUTER");
    outer.handle = doc.allocate_handle();
    let outer_handle = outer.handle;
    doc.block_records.add(outer).unwrap();
    let mut nested = Insert::new("STB_LEAF", Vector3::ZERO);
    nested.common.owner_handle = outer_handle;
    nested.common.plotstyle_flags = 1; // inherit outer INSERT's pen
    doc.add_entity(EntityType::Insert(nested)).unwrap();
    let mut references = Vec::new();
    for (i, style) in [blue, green, blue].into_iter().enumerate() {
        let mut ins = Insert::new("STB_OUTER", Vector3::new(i as f64 * 20.0, 0.0, 0.0));
        ins.common.plotstyle_flags = 3;
        ins.common.plotstyle_handle = Some(style);
        let h = doc.add_entity(EntityType::Insert(ins.clone())).unwrap();
        references.push((ins, h));
    }
    let depths = scene.draw_depth_map();
    let cache = BlockCache::build(
        &scene.document,
        1.0,
        None,
        false,
        [0.0, 0.0, 0.0, 1.0],
        None,
        &depths,
    );
    for ((ins, handle), expected) in references.iter().zip(["Blue", "Green", "Blue"]) {
        let wires = expand_insert(
            &scene.document,
            &cache,
            ins,
            *handle,
            [1.0; 4],
            7,
            0.0,
            [0.0; 8],
            1.0,
            InheritStyle {
                color: [1.0; 4],
                pat_len: 0.0,
                pat: [0.0; 8],
                lw_px: 1.0,
            },
            7,
            true,
            false,
            1.0,
            None,
            None,
            false,
            [0.0, 0.0, 0.0, 1.0],
            1.0,
            BlockScalePolicy::FromInsert,
            false,
            1.0,
        )
        .unwrap();
        let wires = expand_stubs(&wires);
        let inherited = wires
            .iter()
            .find(|wire| wire.points.iter().any(|p| (p[1] - 1.0).abs() < 1e-5))
            .unwrap();
        assert_eq!(
            inherited.effective_plot_style.as_deref(),
            Some(expected),
            "ByBlock must follow this INSERT, including prototype reuse"
        );
        let styles: Vec<_> = wires
            .iter()
            .filter(|wire| !wire.points.is_empty())
            .map(|wire| {
                wire.effective_plot_style
                    .as_deref()
                    .expect("occurrence style")
            })
            .collect();
        assert!(styles.contains(&"Blue"), "explicit child: {styles:?}");
        assert!(styles.contains(&"Green"), "layer-0 ByLayer: {styles:?}");
        assert!(styles.contains(&expected), "nested ByBlock: {styles:?}");
        // At least two separate batches must remain despite identical display properties.
        assert!(styles.len() >= 2, "plot pens must distinguish batches");
    }
}
