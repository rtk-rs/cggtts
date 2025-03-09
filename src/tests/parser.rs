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
    fn parse_dataset() {
        let resources = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("data/CGGTTS");

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
    fn rzsy8257_000() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data/CGGTTS")
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
