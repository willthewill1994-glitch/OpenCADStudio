#[cfg(test)]
mod native_stb_dwg_tests {
    use document::entities::Line;
    use document::objects::{ObjectType, PlaceHolder};
    use document::types::Vector3;
    use document::{DwgWriter, EntityType};
    use OpenCADStudio::io::pdf_export::{
        export_pdf, PdfPageInput, PdfPlotOptions, PlotContent, PlotWire,
    };
    use OpenCADStudio::io::plot_style::{PlotStyleTable, PlotStyleTableExt};

    #[test]
    fn dwg_named_style_handles_reach_native_stb_pdf() {
        let mut doc = document::Document::new();
        doc.header.plotstyle_mode = false;
        let dict = doc.header.acad_plotstylename_dict_handle;
        let mut pen = PlaceHolder::new();
        pen.handle = doc.allocate_handle();
        pen.owner = dict;
        let pen_handle = pen.handle;
        doc.objects.insert(pen_handle, ObjectType::PlaceHolder(pen));
        match doc.objects.get_mut(&dict).unwrap() {
            ObjectType::DictionaryWithDefault(d) => {
                d.entries.push(("Bleu épais".into(), pen_handle))
            }
            _ => panic!("plot style dictionary"),
        }
        doc.layers.get_mut("0").unwrap().plotstyle_handle = pen_handle;
        let mut handles = Vec::new();
        for flags in [0, 1, 2, 3] {
            let mut line = Line::new();
            line.end = Vector3::new(10.0, 10.0, 0.0);
            line.common.plotstyle_flags = flags;
            line.common.plotstyle_handle = (flags == 3).then_some(pen_handle);
            handles.push(doc.add_entity(EntityType::Line(line)).unwrap());
        }
        let bytes = DwgWriter::write_to_vec(&doc).unwrap();
        let mut scene = OpenCADStudio::scene::Scene::new();
        scene.document = OpenCADStudio::io::load_bytes("named.dwg", bytes).unwrap();
        assert!(!scene.document.header.plotstyle_mode);
        let wires: Vec<_> = handles
            .iter()
            .map(|handle| PlotWire {
                wire: OpenCADStudio::scene::WireModel::solid(
                    handle.value().to_string(),
                    vec![[10.0, 10.0, 0.0], [50.0, 50.0, 0.0]],
                    [1.0, 0.0, 0.0, 1.0],
                    false,
                ),
                draw_depth: 0.0,
            })
            .collect(); // ACI 0: true-colour objects must still take named pens.
        let styles: std::collections::HashMap<_, String> = handles
            .iter()
            .map(|handle| {
                let entity = scene.document.get_entity(*handle).unwrap();
                let name = OpenCADStudio::io::plot_style::resolve_entity_plot_style(
                    &scene.document,
                    entity.common(),
                )
                .unwrap();
                (*handle, name.to_string())
            })
            .collect();
        assert_eq!(styles[&handles[0]], "Bleu épais", "ByLayer");
        assert_eq!(styles[&handles[1]], "Normal", "top-level ByBlock");
        assert_eq!(styles[&handles[2]], "Normal");
        assert_eq!(styles[&handles[3]], "Bleu épais", "explicit handle");
        let table = PlotStyleTable::from_bytes(
            "Houle_ech-20à50.stb",
            include_bytes!("fixtures/plot/named-pens.stb"),
        )
        .unwrap();
        assert_eq!(table.style_entry(0).unwrap().name, "Normal");
        let page = PdfPageInput {
            content: PlotContent {
                wires: std::sync::Arc::new(wires),
                entity_plot_styles: styles,
                ..Default::default()
            },
            paper_w: 100.0,
            paper_h: 100.0,
            offset_x: 0.0,
            offset_y: 0.0,
            rotation_deg: 0,
            scale: 1.0,
            clip: None,
            options: PdfPlotOptions::default(),
            plot_style: Some(table),
            geopdf: None,
        };
        let path = std::env::temp_dir().join(format!("ocs-native-stb-{}.pdf", std::process::id()));
        export_pdf(&page, &path).unwrap();
        let pdf = lopdf::Document::load(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        let stream = pdf
            .get_page_content(*pdf.get_pages().values().next().unwrap())
            .unwrap();
        let content = lopdf::content::Content::decode(&stream).unwrap();
        let colors: Vec<_> = content
            .operations
            .iter()
            .filter(|op| op.operator == "RG")
            .map(|op| {
                op.operands
                    .iter()
                    .map(|n| n.as_float().unwrap())
                    .collect::<Vec<_>>()
            })
            .collect();
        assert!(
            colors.contains(&vec![0.0, 0.0, 1.0]),
            "named blue pen: {colors:?}"
        );
        assert!(
            colors.contains(&vec![1.0, 0.0, 0.0]),
            "Normal retains object RGB: {colors:?}"
        );
        assert!(
            content
                .operations
                .iter()
                .any(|op| op.operator == "J" && matches!(op.operands[0].as_i64(), Ok(0))),
            "STB butt cap reaches PDF"
        );
        assert!(
            content.operations.iter().any(|op| op.operator == "w"
                && op.operands[0]
                    .as_float()
                    .is_ok_and(|w| (w - 0.5 * 72.0 / 25.4).abs() < 0.01)),
            "STB physical 0.5 mm pen reaches PDF"
        );
    }
    #[test]
    fn truecolour_fills_use_occurrence_styles_instead_of_pattern_names() {
        use OpenCADStudio::scene::model::hatch_model::{HatchModel, HatchPattern};
        use OpenCADStudio::scene::WireModel;
        let table =
            PlotStyleTable::from_bytes("named.stb", include_bytes!("fixtures/plot/named-pens.stb"))
                .unwrap();
        let mut wire = WireModel::solid("10".into(), Vec::new(), [1.0, 0.0, 0.0, 1.0], false);
        wire.fill_tris = vec![[10.0, 10.0, 0.0], [30.0, 10.0, 0.0], [10.0, 30.0, 0.0]];
        wire.effective_plot_style = Some(std::sync::Arc::from("Bleu épais"));
        let hatch = HatchModel {
            entity: Some(document::Handle::new(11)),
            effective_layer: None,
            effective_plot_style: Some(std::sync::Arc::from("Bleu épais")),
            pattern_origin: None,
            render_instance: None,
            world_origin: [0.0; 2],
            boundary: std::sync::Arc::new(vec![
                [40.0, 10.0],
                [60.0, 10.0],
                [40.0, 30.0],
                [40.0, 10.0],
            ]),
            boundary_wcs: None,
            fill_plane: None,
            fill_plane_boundary: None,
            boundary_exterior: None,
            boundary_sources: None,
            boundary_paths: None,
            style: document::entities::HatchStyleType::Normal,
            pattern: HatchPattern::Solid,
            name: "SOLID".into(),
            color: [1.0, 0.0, 0.0, 1.0],
            aci: 0,
            line_weight_px: 1.0,
            angle_offset: 0.0,
            scale: 1.0,
            draw_depth: 0.0,
        };
        let page = PdfPageInput {
            content: PlotContent {
                wires: std::sync::Arc::new(vec![PlotWire {
                    wire,
                    draw_depth: 0.0,
                }]),
                hatches: vec![hatch],
                entity_plot_styles: [
                    (document::Handle::new(10), "Normal".into()),
                    (document::Handle::new(11), "Normal".into()),
                ]
                .into_iter()
                .collect(),
                ..Default::default()
            },
            paper_w: 100.0,
            paper_h: 100.0,
            offset_x: 0.0,
            offset_y: 0.0,
            rotation_deg: 0,
            scale: 1.0,
            clip: None,
            options: PdfPlotOptions::default(),
            plot_style: Some(table),
            geopdf: None,
        };
        let path = std::env::temp_dir().join(format!("ocs-stb-fills-{}.pdf", std::process::id()));
        export_pdf(&page, &path).unwrap();
        let pdf = lopdf::Document::load(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        let bytes = pdf
            .get_page_content(*pdf.get_pages().values().next().unwrap())
            .unwrap();
        let content = lopdf::content::Content::decode(&bytes).unwrap();
        let colors: Vec<_> = content
            .operations
            .iter()
            .filter(|op| op.operator == "rg")
            .map(|op| {
                op.operands
                    .iter()
                    .map(|n| n.as_float().unwrap())
                    .collect::<Vec<_>>()
            })
            .collect();
        assert!(
            colors.contains(&vec![0.0, 0.0, 1.0]),
            "named blue fills: {colors:?}"
        );
        assert!(
            !colors.contains(&vec![1.0, 0.0, 0.0]),
            "both fills must take their occurrence pen: {colors:?}"
        );
    }
}
