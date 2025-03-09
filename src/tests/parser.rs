mod test {
    use crate::{
        prelude::{Constellation, Epoch, Rcvr, ReferenceTime, CGGTTS, SV},
        tests::toolkit::{cmp_dut_model, random_name},
        Code, Coordinates, Delay,
    };
    use std::fs::File;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    #[test]
    fn single_frequency_files() {
        let resources = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("../data")
            .join("single");
        for entry in std::fs::read_dir(resources).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let is_hidden = path.file_name().unwrap().to_str().unwrap().starts_with('.');
            if is_hidden {
                continue;
            }
            let fp = path.to_string_lossy().to_string();
            println!("parsing \"{}\"", fp);

            let cggtts = CGGTTS::from_file(&fp);
            assert!(
                cggtts.is_ok(),
                "failed to parse {} - {:?}",
                fp,
                cggtts.err()
            );

            let cggtts = cggtts.unwrap();

            // test filename convention
            let stem = path.file_name().unwrap();
            let _stem = stem.to_string_lossy();
            // assert_eq!(cggtts.filename(), stem, "bad filename convention");

            // dump into file
            let filename = random_name(8);
            let mut fd = File::create(&filename).unwrap();
            write!(fd, "{}", cggtts).unwrap();

            // parse back
            let parsed = CGGTTS::from_file(&filename);
            assert!(
                parsed.is_ok(),
                "failed to parse back generated file: {}",
                parsed.err().unwrap()
            );

            println!("running testbench on \"{}\"", filename);
            //TODO: hifitime pb
            // cmp_dut_model(&parsed.unwrap(), &cggtts);

            // remove generated file
            let _ = std::fs::remove_file(&filename);
        }
    }
    #[test]
    fn dual_frequency_files() {
        let resources = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("../data")
            .join("dual");

        for entry in std::fs::read_dir(resources).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let is_hidden = path.file_name().unwrap().to_str().unwrap().starts_with('.');
            if is_hidden {
                continue;
            }
            let fp = path.to_string_lossy().to_string();
            println!("parsing \"{}\"", fp);
            let cggtts = CGGTTS::from_file(&fp);
            assert!(
                cggtts.is_ok(),
                "failed to parse {} - {:?}",
                fp,
                cggtts.err()
            );
        }
    }
    #[test]
    fn gzsy8259_568() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("data")
            .join("single")
            .join("GZSY8259.568");

        let fullpath = path.to_string_lossy().to_string();
        let cggtts = CGGTTS::from_file(&fullpath);
        assert!(cggtts.is_ok());

        let cggtts = cggtts.unwrap();

        assert_eq!(
            cggtts.release_date,
            Epoch::from_gregorian_utc_at_midnight(2014, 02, 20),
        );

        assert_eq!(
            cggtts.rcvr,
            Some(
                Rcvr::default()
                    .manufacturer("GORGYTIMING")
                    .receiver("SYREF25")
                    .serial_number("18259999")
                    .year(2018)
                    .release("v00")
            ),
        );

        assert_eq!(cggtts.station, "SY82");
        assert_eq!(cggtts.nb_channels, 12);
        assert_eq!(cggtts.ims, None);
        assert_eq!(
            cggtts.reference_time,
            ReferenceTime::Custom(String::from("REF(SY82)"))
        );

        assert_eq!(cggtts.reference_frame, Some(String::from("ITRF")));
        assert!((cggtts.apc_coordinates.x - 4314143.824).abs() < 1E-6);
        assert!((cggtts.apc_coordinates.y - 452633.241).abs() < 1E-6);
        assert!((cggtts.apc_coordinates.z - 4660711.385).abs() < 1E-6);
        assert_eq!(cggtts.comments, None);
        assert_eq!(cggtts.tracks.len(), 32);

        let first = cggtts.tracks.first();
        //assert_eq!(cggtts.delay.value(), 0.0);
        assert!(first.is_some());
        let first = first.unwrap();
        assert_eq!(
            first.sv,
            SV {
                constellation: Constellation::GPS,
                prn: 99,
            }
        );

        assert_eq!(cggtts.filename(), String::from("GSSY8259.568"));

        let tracks: Vec<_> = cggtts.tracks().collect();
        assert_eq!(tracks.len(), 32);

        let _dumped = cggtts.to_string();
        let _compare = std::fs::read_to_string(
            env!("CARGO_MANIFEST_DIR").to_owned() + "/../data/single/GZSY8259.568",
        )
        .unwrap();
    }
    #[test]
    fn rzsy8257_000() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("data")
            .join("dual")
            .join("RZSY8257.000");

        let fullpath = path.to_string_lossy().to_string();
        let cggtts = CGGTTS::from_file(&fullpath);
        assert!(cggtts.is_ok());

        let cggtts = cggtts.unwrap();
        assert!(cggtts.rcvr.is_none());
        assert!(cggtts.ims.is_none());
        assert_eq!(
            cggtts.apc_coordinates,
            Coordinates {
                x: 4027881.79,
                y: 306998.67,
                z: 4919499.36,
            }
        );
        assert!(cggtts.comments.is_none());
        assert_eq!(cggtts.station, "ABC");
        assert_eq!(cggtts.nb_channels, 12);

        assert_eq!(cggtts.delay.rf_cable_delay, 237.0);
        assert_eq!(cggtts.delay.ref_delay, 149.6);
        assert_eq!(cggtts.delay.delays.len(), 2);
        assert_eq!(cggtts.delay.delays[0], (Code::C1, Delay::Internal(53.9)));

        let total = cggtts.delay.total_delay(Code::C1);
        assert!(total.is_some());
        assert_eq!(total.unwrap(), 53.9 + 237.0 + 149.6);

        assert_eq!(cggtts.delay.delays[1], (Code::C2, Delay::Internal(49.8)));
        let total = cggtts.delay.total_delay(Code::C2);
        assert!(total.is_some());
        assert_eq!(total.unwrap(), 49.8 + 237.0 + 149.6);

        let cal_id = cggtts.delay.cal_id.clone();
        assert!(cal_id.is_some());
        assert_eq!(cal_id.unwrap(), String::from("1nnn-yyyy"));

        let tracks: Vec<_> = cggtts.tracks().collect();
        assert_eq!(tracks.len(), 4);
    }
}
