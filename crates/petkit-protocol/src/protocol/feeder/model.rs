use petkit_types::FeederDeviceType;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DynamicFeeder;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FreshElementFeeder;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FeederMiniFeeder;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct D3Feeder;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct D4Feeder;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct D4sFeeder;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct D4hFeeder;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct D4shFeeder;

mod model_seal {
    pub trait Sealed {}

    impl Sealed for super::FreshElementFeeder {}
    impl Sealed for super::FeederMiniFeeder {}
    impl Sealed for super::D3Feeder {}
    impl Sealed for super::D4Feeder {}
    impl Sealed for super::D4sFeeder {}
    impl Sealed for super::D4hFeeder {}
    impl Sealed for super::D4shFeeder {}
}

pub trait FeederModel: model_seal::Sealed {
    const DEVICE_TYPE: FeederDeviceType;
}

pub trait SingleHopperFeederModel: FeederModel {
    fn is_valid_single_feed_amount(amount: u16) -> bool;
}

pub trait DualHopperFeederModel: FeederModel {}

pub trait FeederSupportsFoodReplenished: FeederModel {}
pub trait FeederSupportsCalibration: FeederModel {}
pub trait FeederSupportsCallPet: FeederModel {}
pub trait FeederSupportsSound: FeederModel {}
pub trait FeederSupportsCamera: FeederModel {}

macro_rules! feeder_model {
    ($marker:ty, $device_type:expr) => {
        impl FeederModel for $marker {
            const DEVICE_TYPE: FeederDeviceType = $device_type;
        }
    };
}

feeder_model!(FreshElementFeeder, FeederDeviceType::Feeder);
feeder_model!(FeederMiniFeeder, FeederDeviceType::FeederMini);
feeder_model!(D3Feeder, FeederDeviceType::D3);
feeder_model!(D4Feeder, FeederDeviceType::D4);
feeder_model!(D4sFeeder, FeederDeviceType::D4s);
feeder_model!(D4hFeeder, FeederDeviceType::D4h);
feeder_model!(D4shFeeder, FeederDeviceType::D4sh);

impl SingleHopperFeederModel for FreshElementFeeder {
    fn is_valid_single_feed_amount(amount: u16) -> bool {
        (1..=10).contains(&amount)
    }
}

impl SingleHopperFeederModel for FeederMiniFeeder {
    fn is_valid_single_feed_amount(amount: u16) -> bool {
        matches!(amount, 0 | 5 | 10 | 15 | 20 | 25 | 30 | 35 | 40 | 45 | 50)
    }
}

impl SingleHopperFeederModel for D3Feeder {
    fn is_valid_single_feed_amount(amount: u16) -> bool {
        (5..=200).contains(&amount)
    }
}

impl SingleHopperFeederModel for D4Feeder {
    fn is_valid_single_feed_amount(amount: u16) -> bool {
        matches!(amount, 10 | 20 | 30 | 40 | 50)
    }
}

impl SingleHopperFeederModel for D4hFeeder {
    fn is_valid_single_feed_amount(amount: u16) -> bool {
        matches!(amount, 10 | 20 | 30 | 40 | 50)
    }
}

impl DualHopperFeederModel for D4sFeeder {}
impl DualHopperFeederModel for D4shFeeder {}
impl FeederSupportsFoodReplenished for D4sFeeder {}
impl FeederSupportsFoodReplenished for D4hFeeder {}
impl FeederSupportsFoodReplenished for D4shFeeder {}
impl FeederSupportsCalibration for FreshElementFeeder {}
impl FeederSupportsCallPet for D3Feeder {}
impl FeederSupportsSound for D3Feeder {}
impl FeederSupportsSound for D4hFeeder {}
impl FeederSupportsSound for D4shFeeder {}
impl FeederSupportsCamera for D4hFeeder {}
impl FeederSupportsCamera for D4shFeeder {}
