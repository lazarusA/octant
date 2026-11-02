//! `LayerData::empty_volume_plane`: a ring plane emptied for a step not
//! loaded yet is empty in every channel the GPU blends, not only in `volume`.

use super::data::{ChannelVolume, LayerData};
use crate::data::VolumeData;

/// A 2x1 volume of `depth` planes filled with `fill`.
fn volume(depth: usize, fill: f32) -> VolumeData {
    VolumeData::new(2, 1, depth, vec![fill; 2 * depth], 0.0, 1.0, String::new())
}

#[test]
fn an_emptied_plane_is_empty_in_every_channel() {
    const DEPTH: usize = 3;
    const CHANNELS: usize = 2;
    let mut data = LayerData {
        volume: Some(volume(DEPTH, 1.0)),
        channel_volume: Some(ChannelVolume {
            volume: volume(CHANNELS * DEPTH, 1.0),
            c_start: 0,
            channels: CHANNELS,
        }),
        ..Default::default()
    };
    data.empty_volume_plane(1);
    let Some((v, cv)) = data.volume.as_ref().zip(data.channel_volume.as_ref()) else {
        panic!("volumes kept");
    };
    for z in 0..DEPTH {
        let empty = v.values[v.plane_range(z, 1)].iter().all(|x| x.is_nan());
        assert_eq!(empty, z == 1, "volume plane {z}");
        for c in 0..CHANNELS {
            let plane = &cv.volume.values[cv.volume.plane_range(c * DEPTH + z, 1)];
            assert_eq!(
                plane.iter().all(|x| x.is_nan()),
                z == 1,
                "channel {c} plane {z}"
            );
        }
    }
}
