use crate::prelude::{Epoch, Track, TrackData, CGGTTS};
use rand::{distributions::Alphanumeric, Rng};

pub fn cmp_dut_model(dut: &CGGTTS, model: &CGGTTS) {
    assert_eq!(dut.version, model.version, "wrong version");
    assert_eq!(dut.release_date, model.release_date, "wrong release date");
    assert_eq!(dut.station, model.station, "bad station name");
    assert_eq!(dut.rcvr, model.rcvr, "bad receiver data");
    assert_eq!(dut.nb_channels, model.nb_channels, "bad receiver channels");
    assert_eq!(
        dut.reference_time, model.reference_time,
        "bad reference time"
    );
    assert_eq!(
        dut.apc_coordinates, model.apc_coordinates,
        "bad apc coordinates"
    );
    assert_eq!(dut.comments, model.comments, "wrong comments content");
    assert_eq!(dut.delay, model.delay, "bad delay values");

    /* track comparison */
    assert!(
        dut.tracks.len() >= model.tracks.len(),
        "dut is missing some tracks"
    );
    assert!(
        dut.tracks.len() <= model.tracks.len(),
        "dut has too many tracks"
    );
    assert_eq!(
        dut.tracks.len(),
        model.tracks.len(),
        "wrong amount of tracks"
    );

    for (dut_trk, model_trk) in dut.tracks.iter().zip(model.tracks.iter()) {
        cmp_trk_model(dut_trk, model_trk);
    }
}

fn cmp_trk_model(dut_trk: &Track, model_trk: &Track) {
    assert_eq!(dut_trk.epoch, model_trk.epoch, "bad track epoch");
    assert_eq!(
        dut_trk.class, model_trk.class,
        "bad common view class @ {:?}",
        dut_trk.epoch
    );
    assert_eq!(
        dut_trk.duration, model_trk.duration,
        "bad tracking duration @ {:?}",
        dut_trk.epoch
    );
    assert_eq!(
        dut_trk.sv, model_trk.sv,
        "bad sv description @ {:?}",
        dut_trk.epoch
    );
    assert_eq!(
        dut_trk.elevation, model_trk.elevation,
        "bad sv elevation @ {:?}",
        dut_trk.epoch
    );
    assert_eq!(
        dut_trk.azimuth, model_trk.azimuth,
        "bad sv azimuth @ {:?}",
        dut_trk.epoch
    );
    assert_eq!(
        dut_trk.hc, model_trk.hc,
        "bad hardware channel @ {:?}",
        dut_trk.epoch
    );
    assert_eq!(
        dut_trk.fr, model_trk.fr,
        "bad glonass channel @ {:?}",
        dut_trk.epoch
    );
    assert_eq!(
        dut_trk.frc, model_trk.frc,
        "bad carrier code @ {:?}",
        dut_trk.epoch
    );

    trk_data_cmp(dut_trk.epoch, &dut_trk.data, &model_trk.data);
}

pub fn trk_data_cmp(t: Epoch, dut: &TrackData, model: &TrackData) {
    assert_eq!(dut.ioe, model.ioe, "bad IOE @ {:?}", t);
    //assert!((dut.refsv - model.refsv).abs() < 1.0E-9, "bad REFSV @ {:?}");
    //assert!(
    //    (dut.srsv - model.srsv).abs() < 1.0E-9,
    //    "bad SRSV @ {:?} : {} vs {}",
    //    t,
    //    dut.srsv,
    //    model.srsv
    //);
    //assert!((dut.refsys - model.refsys).abs() < 1.0E-9, "bad REFSYS @ {:?}", t);
    //assert!(
    //    (dut.srsys - model.srsys).abs() < 1.0E-9,
    //    "bad SRSYS @ {:?}: {} {}",
    //    t,
    //    dut.srsys,
    //    model.srsys
    //);
    //assert!((dut.dsg - model.dsg).abs() < 1.0E-9, "bad DSG @ {:?}", t);
    //assert!((dut.mdtr - model.mdtr).abs() < 1.0E-9, "bad MDTR @ {:?}", t);
    //assert!((dut.smdt - model.smdt).abs() < 1.0E-9, "bad SMDT @ {:?}", t);
    //assert!((dut.mdio - model.mdio).abs() < 1.0E-9, "bad MDIO @ {:?}", t);
    //assert!((dut.smdi - model.smdi).abs() < 1.0E-9, "bad SMDI @ {:?}", t);
}

/*
 * Tool to generate random names when we need to produce a file
 */
pub fn random_name(size: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}
