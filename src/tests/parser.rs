mod test {

    use hifitime::Unit;

    use crate::{
        header::CalibrationID,
        prelude::CGGTTS,
        tests::toolkit::{random_name, track_dut_model_comparison},
        track::CommonViewClass,
    };
    use std::{
        fs::{read_dir, remove_file},
        path::{Path, PathBuf},
    };

    #[test]
    fn parse_dataset() {
        let dir: PathBuf = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("data/CGGTTS");

        for entry in read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let is_hidden = path.file_name().unwrap().to_str().unwrap().starts_with('.');
            if is_hidden {
                continue;
            }

            let cggtts = CGGTTS::from_file(&path)
                .unwrap_or_else(|e| panic!("failed to parse {}: {}", path.display(), e));

            // Dump then parse back
            let file_name = random_name(8);

            cggtts.to_file(&file_name).unwrap();

            let parsed = CGGTTS::from_file(&file_name)
                .unwrap_or_else(|e| panic!("failed to parse back \"{}\": {}", file_name, e));

            assert_eq!(parsed.tracks.len(), cggtts.tracks.len());

            for (dut, model) in parsed.tracks_iter().zip(cggtts.tracks_iter()) {
                track_dut_model_comparison(dut, model);
            }

            // remove generated file
            let _ = std::fs::remove_file(&file_name);
        }
    }

    #[test]
    fn ezgtr60_258() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data/CGGTTS")
            .join("EZGTR60.258");

        let fullpath = path.to_string_lossy().to_string();

        let cggtts = CGGTTS::from_file(&fullpath).unwrap();

        assert_eq!(cggtts.header.receiver.manufacturer, "GTR51");

        let ims = cggtts.header.ims_hardware.as_ref().unwrap();
        assert_eq!(ims.manufacturer, "GTR51");

        assert!(cggtts.is_galileo_cggtts());

        assert_eq!(
            cggtts.header.delay.calibration_id,
            Some(CalibrationID {
                process_id: 1015,
                year: 2021,
            })
        );

        let mut tests_passed = 0;

        for (nth, track) in cggtts.tracks_iter().enumerate() {
            match nth {
                0 => {
                    assert_eq!(track.sv.to_string(), "E03");
                    assert_eq!(track.class, CommonViewClass::MultiChannel);
                    assert_eq!(track.epoch.to_mjd_utc(Unit::Day).floor() as u16, 60258);
                    assert_eq!(track.duration.to_seconds() as u16, 780);
                    assert!(track.follows_bipm_tracking());
                    assert!((track.elevation_deg - 13.9).abs() < 0.01);
                    assert!((track.azimuth_deg - 54.8).abs() < 0.01);
                    assert!((track.data.refsv - 723788.0 * 0.1E-9) < 1E-10);
                    assert!((track.data.srsv - 14.0 * 0.1E-12) < 1E-12);
                    assert!((track.data.refsys - -302.0 * 0.1E-9) < 1E-10);
                    assert!((track.data.srsys - -14.0 * 0.1E-12) < 1E-12);
                    assert!((track.data.dsg - 2.0 * 0.1E-9).abs() < 1E-10);
                    assert_eq!(track.data.ioe, 76);

                    assert!((track.data.mdtr - 325.0 * 0.1E-9).abs() < 1E-10);
                    assert!((track.data.smdt - -36.0 * 0.1E-12).abs() < 1E-12);
                    assert!((track.data.mdio - 32.0 * 0.1E-9).abs() < 1E-10);
                    assert!((track.data.smdi - -3.0 * 0.1E-12).abs() < 1E-12);

                    let iono = track.iono.unwrap();
                    assert!((iono.msio - 20.0 * 0.1E-9).abs() < 1E-10);
                    assert!((iono.smsi - 20.0 * 0.1E-12).abs() < 1E-12);
                    assert!((iono.isg - 3.0 * 0.1E-9).abs() < 1E-10);

                    assert_eq!(track.frc, "E1");
                    tests_passed += 1;
                },
                _ => {},
            }
        }

        assert_eq!(tests_passed, 1);

        // test filename generator
        assert_eq!(
            cggtts.standardized_file_name(Some("GT"), Some("R")),
            "EZGTR60.258"
        );

        // format (dump) then parse back
        let file_name = random_name(8);
        cggtts.to_file(&file_name).unwrap();

        let parsed = CGGTTS::from_file(&file_name)
            .unwrap_or_else(|e| panic!("failed to parse back CGGTTS \"{}\": {}", file_name, e));

        assert_eq!(parsed.tracks.len(), cggtts.tracks.len());

        for (dut, model) in parsed.tracks_iter().zip(cggtts.tracks_iter()) {
            track_dut_model_comparison(dut, model);
        }

        let _ = remove_file(&file_name);
    }

    #[test]
    fn gzgtr5_60_258() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data/CGGTTS/")
            .join("GZGTR560.258");

        let cggtts = CGGTTS::from_file(&path).unwrap();
        assert_eq!(cggtts.header.receiver.manufacturer, "GTR51");

        let ims = cggtts
            .header
            .ims_hardware
            .as_ref()
            .expect("failed to parse \"IMS=\"");

        assert_eq!(ims.manufacturer, "GTR51");

        assert!(cggtts.is_gps_cggtts());

        assert_eq!(
            cggtts.header.delay.calibration_id,
            Some(CalibrationID {
                process_id: 1015,
                year: 2021,
            })
        );

        let mut tests_passed = 0;

        for (nth, track) in cggtts.tracks_iter().enumerate() {
            match nth {
                0 => {
                    assert_eq!(track.sv.to_string(), "G08");
                    assert_eq!(track.class, CommonViewClass::MultiChannel);
                    assert_eq!(track.epoch.to_mjd_utc(Unit::Day).floor() as u16, 60258);
                    assert_eq!(track.duration.to_seconds() as u16, 780);
                    assert!(track.follows_bipm_tracking());
                    assert!((track.elevation_deg - 24.5).abs() < 0.01);
                    assert!((track.azimuth_deg - 295.4).abs() < 0.01);
                    assert!((track.data.refsv - 1513042.0 * 0.1E-9) < 1E-10);
                    assert!((track.data.srsv - 28.0 * 0.1E-12) < 1E-12);
                    assert!((track.data.refsys - -280.0 * 0.1E-9) < 1E-10);
                    assert!((track.data.srsys - 10.0 * 0.1E-12) < 1E-12);
                    assert!((track.data.dsg - 3.0 * 0.1E-9).abs() < 1E-10);
                    assert_eq!(track.data.ioe, 42);

                    assert!((track.data.mdtr - 192.0 * 0.1E-9).abs() < 1E-10);
                    assert!((track.data.smdt - -49.0 * 0.1E-12).abs() < 1E-12);
                    assert!((track.data.mdio - 99.0 * 0.1E-9).abs() < 1E-10);
                    assert!((track.data.smdi - -14.0 * 0.1E-12).abs() < 1E-12);

                    let iono = track.iono.unwrap();
                    assert!((iono.msio - 57.0 * 0.1E-9).abs() < 1E-10);
                    assert!((iono.smsi - -29.0 * 0.1E-12).abs() < 1E-12);
                    assert!((iono.isg - 5.0 * 0.1E-9).abs() < 1E-10);

                    assert_eq!(track.frc, "L1C");
                    tests_passed += 1;
                },
                _ => {},
            }
        }
        assert_eq!(tests_passed, 1);

        // test filename generator
        assert_eq!(
            cggtts.standardized_file_name(Some("GT"), Some("R5")),
            "GZGTR560.258"
        );

        // format (dump) then parse back
        let file_name = random_name(8);
        cggtts.to_file(&file_name).unwrap();

        let parsed = CGGTTS::from_file(&file_name)
            .unwrap_or_else(|e| panic!("failed to parse back CGGTTS \"{}\": {}", file_name, e));

        assert_eq!(parsed.tracks.len(), cggtts.tracks.len());

        for (dut, model) in parsed.tracks_iter().zip(cggtts.tracks_iter()) {
            track_dut_model_comparison(dut, model);
        }

        let _ = remove_file(&file_name);
    }
}
