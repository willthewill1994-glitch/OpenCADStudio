#[cfg(test)]
mod native_stb_tests {
    use OpenCADStudio::io::plot_style::*;
    const FIXTURE: &[u8] = include_bytes!("fixtures/plot/named-pens.stb");

    #[test]
    fn native_fixture_loads_without_warnings_and_saves_with_native_header() {
        let table = PlotStyleTable::from_bytes("Houle_ech-20à50.stb", FIXTURE).unwrap();
        assert!(table.is_stb);
        assert!(table.load_warnings.is_empty(), "{:?}", table.load_warnings);
        assert_eq!(table.named_entries.len(), 3);
        let blue = with_object_style(&table, "BLEU ÉPAIS").unwrap();
        assert_eq!(
            blue.resolve_object_color(0, [1.0, 0.0, 0.0]),
            Some([0.0, 0.0, 1.0])
        );
        assert_eq!(blue.resolve_style_lineweight(0), Some(0.5));
        assert_eq!(blue.style_entry(0).unwrap().end_style, 0);
        let bytes = encode_stb(&table).unwrap();
        assert!(bytes.starts_with(b"PIAFILEVERSION_2.0,STBVER1,compress"));
        let loaded = PlotStyleTable::from_bytes("reload.stb", &bytes).unwrap();
        assert!(loaded.load_warnings.is_empty());
        assert_eq!(loaded.named_entries["Bleu épais"].color, Some([0, 0, 255]));
        assert!(encode_stb(&PlotStyleTable::identity("color.ctb")).is_err());
    }

    #[test]
    fn named_grayscale_uses_true_object_rgb_and_ctb_leaves_it_alone() {
        let table = PlotStyleTable::from_bytes("pens.stb", FIXTURE).unwrap();
        let gray = with_object_style(&table, "Gris 50").unwrap();
        let actual = gray.resolve_object_color(0, [1.0, 0.0, 0.0]).unwrap();
        assert!((actual[0] - 0.299).abs() < 1e-6);
        assert_eq!(actual, [actual[0]; 3]);
        assert_eq!(gray.resolve_style_screening(0), 0.5);
        let mut color = PlotStyleTable::identity("color.ctb");
        color.aci_entries[0].color = Some([0, 0, 255]);
        assert_eq!(color.resolve_object_color(0, [1.0, 0.0, 0.0]), None);
        assert!(color.style_entry(0).is_none());
    }

    #[test]
    fn missing_normal_never_selects_an_unrelated_named_pen() {
        let mut table = PlotStyleTable::from_bytes("pens.stb", FIXTURE).unwrap();
        table.named_entries.remove("Normal");
        assert!(table.resolve_style_color(1).is_none());
        assert!(table.resolve_style_lineweight(1).is_none());
        assert_eq!(table.resolve_style_screening(1), 1.0);
    }
}
